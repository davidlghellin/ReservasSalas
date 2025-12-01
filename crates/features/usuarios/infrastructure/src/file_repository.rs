use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use usuarios_application::UsuarioRepository;
use usuarios_domain::{Usuario, UsuarioError};

/// Estructura para persistir usuarios en JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UsuariosData {
    usuarios: HashMap<String, Usuario>,
}

/// Adaptador de repositorio que guarda usuarios en un archivo JSON
#[derive(Clone)]
pub struct FileUsuarioRepository {
    /// Path al archivo JSON
    file_path: PathBuf,
    /// Cache en memoria para mejorar rendimiento
    cache: Arc<RwLock<HashMap<String, Usuario>>>,
}

impl FileUsuarioRepository {
    /// Crea un nuevo repositorio de archivo
    ///
    /// # Argumentos
    /// * `file_path` - Ruta al archivo JSON donde se guardarán los usuarios
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Crea un repositorio con ruta por defecto (./data/usuarios.json)
    pub fn default_path() -> Self {
        Self::new(PathBuf::from("./data/usuarios.json"))
    }

    /// Carga los usuarios desde el archivo JSON
    async fn load_from_file(&self) -> Result<(), UsuarioError> {
        // Si el archivo no existe, empezamos con datos vacíos
        if !self.file_path.exists() {
            return Ok(());
        }

        // Leer archivo
        let contents = fs::read_to_string(&self.file_path)
            .await
            .map_err(|e| UsuarioError::ErrorRepositorio(format!("Error al leer archivo: {}", e)))?;

        // Parsear JSON
        let data: UsuariosData = serde_json::from_str(&contents)
            .map_err(|e| UsuarioError::ErrorRepositorio(format!("Error al parsear JSON: {}", e)))?;

        // Actualizar cache
        let mut cache = self.cache.write().await;
        *cache = data.usuarios;

        Ok(())
    }

    /// Guarda los usuarios en el archivo JSON
    async fn save_to_file(&self) -> Result<(), UsuarioError> {
        // Crear directorio si no existe
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                UsuarioError::ErrorRepositorio(format!("Error al crear directorio: {}", e))
            })?;
        }

        // Leer cache
        let usuarios = {
            let cache = self.cache.read().await;
            cache.clone()
        };

        // Preparar datos
        let data = UsuariosData { usuarios };

        // Serializar a JSON (pretty print)
        let json = serde_json::to_string_pretty(&data).map_err(|e| {
            UsuarioError::ErrorRepositorio(format!("Error al serializar JSON: {}", e))
        })?;

        // Escribir al archivo
        fs::write(&self.file_path, json).await.map_err(|e| {
            UsuarioError::ErrorRepositorio(format!("Error al escribir archivo: {}", e))
        })?;

        Ok(())
    }

    /// Inicializa el repositorio cargando datos del archivo
    ///
    /// Llamar este método al inicio para cargar usuarios existentes
    pub async fn init(&self) -> Result<(), UsuarioError> {
        self.load_from_file().await
    }
}

#[async_trait]
impl UsuarioRepository for FileUsuarioRepository {
    async fn guardar(&self, usuario: &Usuario) -> Result<(), UsuarioError> {
        // Guardar en cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(usuario.id.clone(), usuario.clone());
        }

        // Persistir a disco
        self.save_to_file().await?;

        Ok(())
    }

    async fn obtener(&self, id: &str) -> Result<Option<Usuario>, UsuarioError> {
        let cache = self.cache.read().await;
        Ok(cache.get(id).cloned())
    }

    async fn obtener_por_email(&self, email: &str) -> Result<Option<Usuario>, UsuarioError> {
        let cache = self.cache.read().await;
        Ok(cache.values().find(|u| u.email == email).cloned())
    }

    async fn listar(&self) -> Result<Vec<Usuario>, UsuarioError> {
        let cache = self.cache.read().await;
        Ok(cache.values().cloned().collect())
    }

    async fn actualizar(&self, usuario: &Usuario) -> Result<(), UsuarioError> {
        // Actualizar en cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(usuario.id.clone(), usuario.clone());
        }

        // Persistir a disco
        self.save_to_file().await?;

        Ok(())
    }

    async fn eliminar(&self, id: &str) -> Result<(), UsuarioError> {
        // Eliminar de cache
        {
            let mut cache = self.cache.write().await;
            cache.remove(id);
        }

        // Persistir a disco
        self.save_to_file().await?;

        Ok(())
    }

    async fn existe_email(&self, email: &str) -> Result<bool, UsuarioError> {
        let cache = self.cache.read().await;
        Ok(cache.values().any(|u| u.email == email))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use usuarios_auth::PasswordService;
    use usuarios_domain::Rol;

    async fn crear_repo_temporal() -> (FileUsuarioRepository, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_usuarios.json");
        let repo = FileUsuarioRepository::new(file_path);
        repo.init().await.unwrap();
        (repo, temp_dir)
    }

    #[tokio::test]
    async fn test_guardar_y_obtener_usuario() {
        let (repo, _temp) = crear_repo_temporal().await;

        let hash = PasswordService::hash_password("password123").unwrap();
        let usuario = Usuario::new(
            "Test User".to_string(),
            "test@example.com".to_string(),
            hash,
            Rol::Usuario,
        )
        .unwrap();

        // Guardar
        repo.guardar(&usuario).await.unwrap();

        // Obtener por ID
        let resultado = repo.obtener(&usuario.id).await.unwrap();
        assert!(resultado.is_some());
        assert_eq!(resultado.unwrap().email, "test@example.com");
    }

    #[tokio::test]
    async fn test_obtener_por_email() {
        let (repo, _temp) = crear_repo_temporal().await;

        let hash = PasswordService::hash_password("password").unwrap();
        let usuario = Usuario::new(
            "Email User".to_string(),
            "email@test.com".to_string(),
            hash,
            Rol::Usuario,
        )
        .unwrap();

        repo.guardar(&usuario).await.unwrap();

        // Obtener por email
        let resultado = repo.obtener_por_email("email@test.com").await.unwrap();
        assert!(resultado.is_some());
        assert_eq!(resultado.unwrap().nombre, "Email User");
    }

    #[tokio::test]
    async fn test_listar_usuarios() {
        let (repo, _temp) = crear_repo_temporal().await;

        let hash1 = PasswordService::hash_password("pass1").unwrap();
        let hash2 = PasswordService::hash_password("pass2").unwrap();

        let user1 = Usuario::new(
            "User 1".to_string(),
            "user1@test.com".to_string(),
            hash1,
            Rol::Usuario,
        )
        .unwrap();

        let user2 = Usuario::new(
            "User 2".to_string(),
            "user2@test.com".to_string(),
            hash2,
            Rol::Admin,
        )
        .unwrap();

        repo.guardar(&user1).await.unwrap();
        repo.guardar(&user2).await.unwrap();

        let usuarios = repo.listar().await.unwrap();
        assert_eq!(usuarios.len(), 2);
    }

    #[tokio::test]
    async fn test_actualizar_usuario() {
        let (repo, _temp) = crear_repo_temporal().await;

        let hash = PasswordService::hash_password("password").unwrap();
        let mut usuario = Usuario::new(
            "Original Name".to_string(),
            "update@test.com".to_string(),
            hash,
            Rol::Usuario,
        )
        .unwrap();

        repo.guardar(&usuario).await.unwrap();

        // Actualizar nombre
        usuario.actualizar_nombre("New Name".to_string()).unwrap();
        repo.actualizar(&usuario).await.unwrap();

        // Verificar cambio
        let resultado = repo.obtener(&usuario.id).await.unwrap().unwrap();
        assert_eq!(resultado.nombre, "New Name");
    }

    #[tokio::test]
    async fn test_eliminar_usuario() {
        let (repo, _temp) = crear_repo_temporal().await;

        let hash = PasswordService::hash_password("password").unwrap();
        let usuario = Usuario::new(
            "Delete Me".to_string(),
            "delete@test.com".to_string(),
            hash,
            Rol::Usuario,
        )
        .unwrap();

        repo.guardar(&usuario).await.unwrap();

        // Verificar que existe
        assert!(repo.obtener(&usuario.id).await.unwrap().is_some());

        // Eliminar
        repo.eliminar(&usuario.id).await.unwrap();

        // Verificar que no existe
        assert!(repo.obtener(&usuario.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_existe_email() {
        let (repo, _temp) = crear_repo_temporal().await;

        let hash = PasswordService::hash_password("password").unwrap();
        let usuario = Usuario::new(
            "Exists User".to_string(),
            "exists@test.com".to_string(),
            hash,
            Rol::Usuario,
        )
        .unwrap();

        // No existe antes de guardar
        assert!(!repo.existe_email("exists@test.com").await.unwrap());

        repo.guardar(&usuario).await.unwrap();

        // Existe después de guardar
        assert!(repo.existe_email("exists@test.com").await.unwrap());
    }

    #[tokio::test]
    async fn test_persistencia_en_archivo() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("persistencia.json");

        let user_id;

        // Crear repo y guardar usuario
        {
            let repo = FileUsuarioRepository::new(file_path.clone());
            repo.init().await.unwrap();

            let hash = PasswordService::hash_password("password").unwrap();
            let usuario = Usuario::new(
                "Persistent User".to_string(),
                "persistent@test.com".to_string(),
                hash,
                Rol::Usuario,
            )
            .unwrap();

            user_id = usuario.id.clone();
            repo.guardar(&usuario).await.unwrap();
        }

        // Verificar que el archivo existe
        assert!(file_path.exists());

        // Crear nuevo repo y cargar desde archivo
        {
            let repo = FileUsuarioRepository::new(file_path.clone());
            repo.init().await.unwrap();

            let usuario = repo.obtener(&user_id).await.unwrap();
            assert!(usuario.is_some());
            assert_eq!(usuario.unwrap().nombre, "Persistent User");
        }
    }

    #[tokio::test]
    async fn test_archivo_json_formato_correcto() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("formato.json");

        let repo = FileUsuarioRepository::new(file_path.clone());
        repo.init().await.unwrap();

        let hash = PasswordService::hash_password("password").unwrap();
        let usuario = Usuario::new(
            "JSON User".to_string(),
            "json@test.com".to_string(),
            hash,
            Rol::Admin,
        )
        .unwrap();

        let user_id = usuario.id.clone();
        repo.guardar(&usuario).await.unwrap();

        // Leer archivo y verificar estructura JSON
        let contents = std::fs::read_to_string(&file_path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&contents).unwrap();

        assert!(json.get("usuarios").is_some());
        assert!(json["usuarios"].get(&user_id).is_some());
        assert_eq!(json["usuarios"][&user_id]["nombre"], "JSON User");
        assert_eq!(json["usuarios"][&user_id]["email"], "json@test.com");
        assert_eq!(json["usuarios"][&user_id]["rol"], "Admin");
    }
}
