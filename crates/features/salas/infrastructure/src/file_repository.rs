use async_trait::async_trait;
use salas_application::SalaRepository;
use salas_domain::{Sala, SalaError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

/// Estructura para persistir las salas en JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SalasData {
    salas: HashMap<String, Sala>,
}

/// Adaptador de repositorio que guarda las salas en un archivo JSON
#[derive(Clone)]
pub struct FileSalaRepository {
    /// Path al archivo JSON donde se guardan las salas
    file_path: PathBuf,
    /// Cache en memoria para mejorar rendimiento
    cache: Arc<RwLock<HashMap<String, Sala>>>,
}

impl FileSalaRepository {
    /// Crea un nuevo repositorio de fichero
    ///
    /// # Argumentos
    /// * `file_path` - Ruta al archivo JSON donde se guardarán las salas
    ///
    /// # Ejemplo
    /// ```rust
    /// use std::path::PathBuf;
    /// use salas_infrastructure::FileSalaRepository;
    ///
    /// let repo = FileSalaRepository::new(PathBuf::from("salas.json"));
    /// ```
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Crea un repositorio con ruta por defecto (./data/salas.json)
    pub fn default_path() -> Self {
        Self::new(PathBuf::from("./data/salas.json"))
    }

    /// Carga las salas desde el archivo JSON
    async fn load_from_file(&self) -> Result<(), SalaError> {
        // Si el archivo no existe, no es un error (empezamos con datos vacíos)
        if !self.file_path.exists() {
            return Ok(());
        }

        // Leer el archivo
        let contents = fs::read_to_string(&self.file_path)
            .await
            .map_err(|e| SalaError::ErrorRepositorio(format!("Error al leer archivo: {}", e)))?;

        // Parsear JSON
        let data: SalasData = serde_json::from_str(&contents)
            .map_err(|e| SalaError::ErrorRepositorio(format!("Error al parsear JSON: {}", e)))?;

        // Actualizar cache
        let mut cache = self.cache.write().await;
        *cache = data.salas;

        Ok(())
    }

    /// Guarda las salas en el archivo JSON
    async fn save_to_file(&self) -> Result<(), SalaError> {
        // Crear directorio si no existe
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                SalaError::ErrorRepositorio(format!("Error al crear directorio: {}", e))
            })?;
        }

        // Leer cache y clonar datos
        let salas = {
            let cache = self.cache.read().await;
            cache.clone()
        };

        // Preparar datos para serializar
        let data = SalasData { salas };

        // Serializar a JSON (pretty print)
        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| SalaError::ErrorRepositorio(format!("Error al serializar JSON: {}", e)))?;

        // Escribir al archivo
        fs::write(&self.file_path, json).await.map_err(|e| {
            SalaError::ErrorRepositorio(format!("Error al escribir archivo: {}", e))
        })?;

        Ok(())
    }

    /// Inicializa el repositorio cargando datos del archivo
    ///
    /// Llamar este método al inicio para cargar las salas existentes
    pub async fn init(&self) -> Result<(), SalaError> {
        self.load_from_file().await
    }
}

#[async_trait]
impl SalaRepository for FileSalaRepository {
    async fn guardar(&self, sala: &Sala) -> Result<(), SalaError> {
        // Guardar en cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(sala.id.clone(), sala.clone());
        }

        // Persistir a disco
        self.save_to_file().await?;

        Ok(())
    }

    async fn obtener(&self, id: &str) -> Result<Option<Sala>, SalaError> {
        let cache = self.cache.read().await;
        Ok(cache.get(id).cloned())
    }

    async fn listar(&self) -> Result<Vec<Sala>, SalaError> {
        let cache = self.cache.read().await;
        Ok(cache.values().cloned().collect())
    }

    async fn actualizar(&self, sala: &Sala) -> Result<(), SalaError> {
        // Actualizar en cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(sala.id.clone(), sala.clone());
        }

        // Persistir a disco
        self.save_to_file().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    async fn crear_repo_temporal() -> (FileSalaRepository, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_salas.json");
        let repo = FileSalaRepository::new(file_path);
        repo.init().await.unwrap();
        (repo, temp_dir)
    }

    #[tokio::test]
    async fn test_guardar_y_obtener_sala() {
        let (repo, _temp) = crear_repo_temporal().await;

        let sala = Sala::new("123".to_string(), "Sala Test".to_string(), 50).unwrap();

        // Guardar
        repo.guardar(&sala).await.unwrap();

        // Obtener
        let resultado = repo.obtener("123").await.unwrap();
        assert!(resultado.is_some());
        assert_eq!(resultado.unwrap().nombre, "Sala Test");
    }

    #[tokio::test]
    async fn test_listar_salas() {
        let (repo, _temp) = crear_repo_temporal().await;

        let sala1 = Sala::new("1".to_string(), "Sala 1".to_string(), 10).unwrap();
        let sala2 = Sala::new("2".to_string(), "Sala 2".to_string(), 20).unwrap();

        repo.guardar(&sala1).await.unwrap();
        repo.guardar(&sala2).await.unwrap();

        let salas = repo.listar().await.unwrap();
        assert_eq!(salas.len(), 2);
    }

    #[tokio::test]
    async fn test_actualizar_sala() {
        let (repo, _temp) = crear_repo_temporal().await;

        let mut sala = Sala::new("123".to_string(), "Sala Original".to_string(), 50).unwrap();
        repo.guardar(&sala).await.unwrap();

        // Modificar y actualizar
        sala.desactivar();
        repo.actualizar(&sala).await.unwrap();

        // Verificar cambio
        let resultado = repo.obtener("123").await.unwrap().unwrap();
        assert!(!resultado.esta_activa());
    }

    #[tokio::test]
    async fn test_persistencia_en_archivo() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("persistencia.json");

        // Crear repo y guardar sala
        {
            let repo = FileSalaRepository::new(file_path.clone());
            repo.init().await.unwrap();

            let sala = Sala::new("123".to_string(), "Sala Persistente".to_string(), 30).unwrap();
            repo.guardar(&sala).await.unwrap();
        }

        // Verificar que el archivo existe
        assert!(file_path.exists());

        // Crear nuevo repo y cargar desde archivo
        {
            let repo = FileSalaRepository::new(file_path.clone());
            repo.init().await.unwrap();

            let sala = repo.obtener("123").await.unwrap();
            assert!(sala.is_some());
            assert_eq!(sala.unwrap().nombre, "Sala Persistente");
        }
    }

    #[tokio::test]
    async fn test_archivo_json_formato_correcto() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("formato.json");

        let repo = FileSalaRepository::new(file_path.clone());
        repo.init().await.unwrap();

        let sala = Sala::new("abc-123".to_string(), "Sala JSON".to_string(), 25).unwrap();
        repo.guardar(&sala).await.unwrap();

        // Leer archivo y verificar estructura JSON
        let contents = fs::read_to_string(&file_path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&contents).unwrap();

        assert!(json.get("salas").is_some());
        assert!(json["salas"].get("abc-123").is_some());
        assert_eq!(json["salas"]["abc-123"]["nombre"], "Sala JSON");
    }
}
