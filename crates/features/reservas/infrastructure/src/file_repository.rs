use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reservas_application::ReservaRepository;
use reservas_domain::{Reserva, ReservaError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

/// Estructura para persistir reservas en JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReservasData {
    reservas: HashMap<String, Reserva>,
}

/// Adaptador de repositorio que guarda reservas en un archivo JSON
#[derive(Clone)]
pub struct FileReservaRepository {
    /// Path al archivo JSON
    file_path: PathBuf,
    /// Cache en memoria para mejorar rendimiento
    cache: Arc<RwLock<HashMap<String, Reserva>>>,
}

impl FileReservaRepository {
    /// Crea un nuevo repositorio de archivo
    ///
    /// # Argumentos
    /// * `file_path` - Ruta al archivo JSON donde se guardarán las reservas
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Crea un repositorio con ruta por defecto (./data/reservas.json)
    pub fn default_path() -> Self {
        Self::new(PathBuf::from("./data/reservas.json"))
    }

    /// Carga las reservas desde el archivo JSON
    async fn load_from_file(&self) -> Result<(), ReservaError> {
        // Si el archivo no existe, empezamos con datos vacíos
        if !self.file_path.exists() {
            return Ok(());
        }

        // Leer archivo
        let contents = fs::read_to_string(&self.file_path)
            .await
            .map_err(|e| ReservaError::ErrorRepositorio(format!("Error al leer archivo: {}", e)))?;

        // Parsear JSON
        let data: ReservasData = serde_json::from_str(&contents).map_err(|e| {
            ReservaError::ErrorRepositorio(format!("Error al parsear JSON: {}", e))
        })?;

        // Actualizar cache
        let mut cache = self.cache.write().await;
        *cache = data.reservas;

        Ok(())
    }

    /// Guarda las reservas en el archivo JSON
    async fn save_to_file(&self) -> Result<(), ReservaError> {
        // Crear directorio si no existe
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                ReservaError::ErrorRepositorio(format!("Error al crear directorio: {}", e))
            })?;
        }

        // Leer cache
        let reservas = {
            let cache = self.cache.read().await;
            cache.clone()
        };

        // Preparar datos
        let data = ReservasData { reservas };

        // Serializar a JSON (pretty print)
        let json = serde_json::to_string_pretty(&data).map_err(|e| {
            ReservaError::ErrorRepositorio(format!("Error al serializar JSON: {}", e))
        })?;

        // Escribir al archivo
        fs::write(&self.file_path, json)
            .await
            .map_err(|e| {
                ReservaError::ErrorRepositorio(format!("Error al escribir archivo: {}", e))
            })?;

        Ok(())
    }

    /// Inicializa el repositorio cargando datos del archivo
    ///
    /// Llamar este método al inicio para cargar reservas existentes
    pub async fn init(&self) -> Result<(), ReservaError> {
        self.load_from_file().await
    }

    /// Obtiene el número de reservas almacenadas
    pub async fn count(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }
}

#[async_trait]
impl ReservaRepository for FileReservaRepository {
    async fn guardar(&self, reserva: &Reserva) -> Result<(), ReservaError> {
        let mut cache = self.cache.write().await;
        cache.insert(reserva.id().to_string(), reserva.clone());
        drop(cache); // Liberar el lock antes de guardar al archivo

        self.save_to_file().await?;
        Ok(())
    }

    async fn obtener(&self, id: &str) -> Result<Option<Reserva>, ReservaError> {
        let cache = self.cache.read().await;
        Ok(cache.get(id).cloned())
    }

    async fn listar(&self) -> Result<Vec<Reserva>, ReservaError> {
        let cache = self.cache.read().await;
        Ok(cache.values().cloned().collect())
    }

    async fn listar_por_sala(&self, sala_id: &str) -> Result<Vec<Reserva>, ReservaError> {
        let cache = self.cache.read().await;
        Ok(cache
            .values()
            .filter(|r| r.sala_id() == sala_id)
            .cloned()
            .collect())
    }

    async fn listar_por_usuario(&self, usuario_id: &str) -> Result<Vec<Reserva>, ReservaError> {
        let cache = self.cache.read().await;
        Ok(cache
            .values()
            .filter(|r| r.usuario_id() == usuario_id)
            .cloned()
            .collect())
    }

    async fn listar_por_sala_y_rango(
        &self,
        sala_id: &str,
        inicio: DateTime<Utc>,
        fin: DateTime<Utc>,
    ) -> Result<Vec<Reserva>, ReservaError> {
        let cache = self.cache.read().await;
        Ok(cache
            .values()
            .filter(|r| {
                r.sala_id() == sala_id
                    && r.esta_activa()
                    && r.fecha_inicio() < fin
                    && r.fecha_fin() > inicio
            })
            .cloned()
            .collect())
    }

    async fn actualizar(&self, reserva: &Reserva) -> Result<(), ReservaError> {
        let mut cache = self.cache.write().await;

        if cache.contains_key(reserva.id()) {
            cache.insert(reserva.id().to_string(), reserva.clone());
            drop(cache); // Liberar el lock antes de guardar al archivo

            self.save_to_file().await?;
            Ok(())
        } else {
            Err(ReservaError::NoEncontrada)
        }
    }

    async fn eliminar(&self, id: &str) -> Result<(), ReservaError> {
        let mut cache = self.cache.write().await;

        if cache.remove(id).is_some() {
            drop(cache); // Liberar el lock antes de guardar al archivo

            self.save_to_file().await?;
            Ok(())
        } else {
            Err(ReservaError::NoEncontrada)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use reservas_domain::EstadoReserva;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_guardar_y_cargar_desde_archivo() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("reservas_test.json");

        let repo = FileReservaRepository::new(file_path.clone());

        let ahora = Utc::now();
        let inicio = ahora + Duration::hours(1);
        let fin = inicio + Duration::hours(2);

        let reserva = Reserva::new("sala1".into(), "usuario1".into(), inicio, fin).unwrap();
        let id = reserva.id().to_string();

        // Guardar reserva
        repo.guardar(&reserva).await.unwrap();

        // Verificar que el archivo existe
        assert!(file_path.exists());

        // Crear nuevo repositorio y cargar desde archivo
        let repo2 = FileReservaRepository::new(file_path.clone());
        repo2.init().await.unwrap();

        // Verificar que la reserva se cargó correctamente
        let obtenida = repo2.obtener(&id).await.unwrap();
        assert!(obtenida.is_some());
        assert_eq!(obtenida.unwrap().id(), id);
    }

    #[tokio::test]
    async fn test_persistencia_multiple_reservas() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("reservas_test.json");

        let repo = FileReservaRepository::new(file_path.clone());

        let ahora = Utc::now();

        let r1 = Reserva::new(
            "sala1".into(),
            "usuario1".into(),
            ahora + Duration::hours(1),
            ahora + Duration::hours(2),
        )
        .unwrap();

        let r2 = Reserva::new(
            "sala2".into(),
            "usuario2".into(),
            ahora + Duration::hours(3),
            ahora + Duration::hours(4),
        )
        .unwrap();

        repo.guardar(&r1).await.unwrap();
        repo.guardar(&r2).await.unwrap();

        // Crear nuevo repositorio y cargar
        let repo2 = FileReservaRepository::new(file_path.clone());
        repo2.init().await.unwrap();

        let todas = repo2.listar().await.unwrap();
        assert_eq!(todas.len(), 2);
    }

    #[tokio::test]
    async fn test_actualizar_persiste_cambios() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("reservas_test.json");

        let repo = FileReservaRepository::new(file_path.clone());

        let ahora = Utc::now();
        let inicio = ahora + Duration::hours(1);
        let fin = inicio + Duration::hours(2);

        let mut reserva = Reserva::new("sala1".into(), "usuario1".into(), inicio, fin).unwrap();
        let id = reserva.id().to_string();

        repo.guardar(&reserva).await.unwrap();

        // Cancelar reserva
        reserva.cancelar();
        repo.actualizar(&reserva).await.unwrap();

        // Cargar desde archivo en nuevo repositorio
        let repo2 = FileReservaRepository::new(file_path.clone());
        repo2.init().await.unwrap();

        let obtenida = repo2.obtener(&id).await.unwrap().unwrap();
        assert_eq!(obtenida.estado(), &EstadoReserva::Cancelada);
    }

    #[tokio::test]
    async fn test_eliminar_persiste_cambios() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("reservas_test.json");

        let repo = FileReservaRepository::new(file_path.clone());

        let ahora = Utc::now();
        let inicio = ahora + Duration::hours(1);
        let fin = inicio + Duration::hours(2);

        let reserva = Reserva::new("sala1".into(), "usuario1".into(), inicio, fin).unwrap();
        let id = reserva.id().to_string();

        repo.guardar(&reserva).await.unwrap();
        assert_eq!(repo.count().await, 1);

        repo.eliminar(&id).await.unwrap();
        assert_eq!(repo.count().await, 0);

        // Cargar desde archivo en nuevo repositorio
        let repo2 = FileReservaRepository::new(file_path.clone());
        repo2.init().await.unwrap();

        assert_eq!(repo2.count().await, 0);
    }

    #[tokio::test]
    async fn test_listar_por_sala() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("reservas_test.json");

        let repo = FileReservaRepository::new(file_path);

        let ahora = Utc::now();

        let r1 = Reserva::new(
            "sala1".into(),
            "usuario1".into(),
            ahora + Duration::hours(1),
            ahora + Duration::hours(2),
        )
        .unwrap();

        let r2 = Reserva::new(
            "sala1".into(),
            "usuario2".into(),
            ahora + Duration::hours(3),
            ahora + Duration::hours(4),
        )
        .unwrap();

        let r3 = Reserva::new(
            "sala2".into(),
            "usuario1".into(),
            ahora + Duration::hours(1),
            ahora + Duration::hours(2),
        )
        .unwrap();

        repo.guardar(&r1).await.unwrap();
        repo.guardar(&r2).await.unwrap();
        repo.guardar(&r3).await.unwrap();

        let sala1_reservas = repo.listar_por_sala("sala1").await.unwrap();
        assert_eq!(sala1_reservas.len(), 2);

        let sala2_reservas = repo.listar_por_sala("sala2").await.unwrap();
        assert_eq!(sala2_reservas.len(), 1);
    }

    #[tokio::test]
    async fn test_listar_por_usuario() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("reservas_test.json");

        let repo = FileReservaRepository::new(file_path);

        let ahora = Utc::now();

        let r1 = Reserva::new(
            "sala1".into(),
            "usuario1".into(),
            ahora + Duration::hours(1),
            ahora + Duration::hours(2),
        )
        .unwrap();

        let r2 = Reserva::new(
            "sala2".into(),
            "usuario1".into(),
            ahora + Duration::hours(3),
            ahora + Duration::hours(4),
        )
        .unwrap();

        let r3 = Reserva::new(
            "sala1".into(),
            "usuario2".into(),
            ahora + Duration::hours(5),
            ahora + Duration::hours(6),
        )
        .unwrap();

        repo.guardar(&r1).await.unwrap();
        repo.guardar(&r2).await.unwrap();
        repo.guardar(&r3).await.unwrap();

        let usuario1_reservas = repo.listar_por_usuario("usuario1").await.unwrap();
        assert_eq!(usuario1_reservas.len(), 2);

        let usuario2_reservas = repo.listar_por_usuario("usuario2").await.unwrap();
        assert_eq!(usuario2_reservas.len(), 1);
    }

    #[tokio::test]
    async fn test_listar_por_sala_y_rango() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("reservas_test.json");

        let repo = FileReservaRepository::new(file_path);

        let ahora = Utc::now();

        // Reserva en sala1 de 1h a 3h
        let r1 = Reserva::new(
            "sala1".into(),
            "usuario1".into(),
            ahora + Duration::hours(1),
            ahora + Duration::hours(3),
        )
        .unwrap();

        // Reserva en sala1 de 5h a 7h
        let r2 = Reserva::new(
            "sala1".into(),
            "usuario2".into(),
            ahora + Duration::hours(5),
            ahora + Duration::hours(7),
        )
        .unwrap();

        repo.guardar(&r1).await.unwrap();
        repo.guardar(&r2).await.unwrap();

        // Buscar en sala1 de 0h a 4h (debe encontrar r1)
        let reservas = repo
            .listar_por_sala_y_rango("sala1", ahora, ahora + Duration::hours(4))
            .await
            .unwrap();
        assert_eq!(reservas.len(), 1);

        // Buscar en sala1 de 4h a 8h (debe encontrar r2)
        let reservas = repo
            .listar_por_sala_y_rango(
                "sala1",
                ahora + Duration::hours(4),
                ahora + Duration::hours(8),
            )
            .await
            .unwrap();
        assert_eq!(reservas.len(), 1);
    }

    #[tokio::test]
    async fn test_archivo_no_existe_al_inicio() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("no_existe.json");

        let repo = FileReservaRepository::new(file_path.clone());
        repo.init().await.unwrap();

        // No debería haber error al iniciar con archivo inexistente
        assert_eq!(repo.count().await, 0);
    }
}
