use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reservas_application::ReservaRepository;
use reservas_domain::{Reserva, ReservaError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Repositorio en memoria para Reservas
/// Útil para testing y desarrollo
#[derive(Clone)]
pub struct InMemoryReservaRepository {
    reservas: Arc<RwLock<HashMap<String, Reserva>>>,
}

impl InMemoryReservaRepository {
    /// Crea un nuevo repositorio en memoria vacío
    pub fn new() -> Self {
        Self {
            reservas: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Crea un repositorio con reservas iniciales
    pub fn with_reservas(reservas: Vec<Reserva>) -> Self {
        let mut map = HashMap::new();
        for reserva in reservas {
            map.insert(reserva.id().to_string(), reserva);
        }

        Self {
            reservas: Arc::new(RwLock::new(map)),
        }
    }

    /// Limpia todas las reservas (útil para testing)
    pub async fn clear(&self) {
        let mut reservas = self.reservas.write().await;
        reservas.clear();
    }

    /// Obtiene el número de reservas almacenadas
    pub async fn count(&self) -> usize {
        let reservas = self.reservas.read().await;
        reservas.len()
    }
}

impl Default for InMemoryReservaRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ReservaRepository for InMemoryReservaRepository {
    async fn guardar(&self, reserva: &Reserva) -> Result<(), ReservaError> {
        let mut reservas = self.reservas.write().await;
        reservas.insert(reserva.id().to_string(), reserva.clone());
        Ok(())
    }

    async fn obtener(&self, id: &str) -> Result<Option<Reserva>, ReservaError> {
        let reservas = self.reservas.read().await;
        Ok(reservas.get(id).cloned())
    }

    async fn listar(&self) -> Result<Vec<Reserva>, ReservaError> {
        let reservas = self.reservas.read().await;
        Ok(reservas.values().cloned().collect())
    }

    async fn listar_por_sala(&self, sala_id: &str) -> Result<Vec<Reserva>, ReservaError> {
        let reservas = self.reservas.read().await;
        Ok(reservas
            .values()
            .filter(|r| r.sala_id() == sala_id)
            .cloned()
            .collect())
    }

    async fn listar_por_usuario(&self, usuario_id: &str) -> Result<Vec<Reserva>, ReservaError> {
        let reservas = self.reservas.read().await;
        Ok(reservas
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
        let reservas = self.reservas.read().await;
        Ok(reservas
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
        let mut reservas = self.reservas.write().await;

        if reservas.contains_key(reserva.id()) {
            reservas.insert(reserva.id().to_string(), reserva.clone());
            Ok(())
        } else {
            Err(ReservaError::NoEncontrada)
        }
    }

    async fn eliminar(&self, id: &str) -> Result<(), ReservaError> {
        let mut reservas = self.reservas.write().await;

        if reservas.remove(id).is_some() {
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

    #[tokio::test]
    async fn test_crear_y_obtener_reserva() {
        let repo = InMemoryReservaRepository::new();

        let ahora = Utc::now();
        let inicio = ahora + Duration::hours(1);
        let fin = inicio + Duration::hours(2);

        let reserva = Reserva::new("sala1".into(), "usuario1".into(), inicio, fin).unwrap();
        let id = reserva.id().to_string();

        repo.guardar(&reserva).await.unwrap();

        let obtenida = repo.obtener(&id).await.unwrap();
        assert!(obtenida.is_some());
        assert_eq!(obtenida.unwrap().id(), id);
    }

    #[tokio::test]
    async fn test_listar_reservas() {
        let repo = InMemoryReservaRepository::new();

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

        let todas = repo.listar().await.unwrap();
        assert_eq!(todas.len(), 2);
    }

    #[tokio::test]
    async fn test_listar_por_sala() {
        let repo = InMemoryReservaRepository::new();

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
        let repo = InMemoryReservaRepository::new();

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
        let repo = InMemoryReservaRepository::new();

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

        // Reserva en sala2 de 1h a 3h
        let r3 = Reserva::new(
            "sala2".into(),
            "usuario1".into(),
            ahora + Duration::hours(1),
            ahora + Duration::hours(3),
        )
        .unwrap();

        repo.guardar(&r1).await.unwrap();
        repo.guardar(&r2).await.unwrap();
        repo.guardar(&r3).await.unwrap();

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

        // Buscar en sala1 de 0h a 8h (debe encontrar r1 y r2)
        let reservas = repo
            .listar_por_sala_y_rango("sala1", ahora, ahora + Duration::hours(8))
            .await
            .unwrap();
        assert_eq!(reservas.len(), 2);
    }

    #[tokio::test]
    async fn test_actualizar_reserva() {
        let repo = InMemoryReservaRepository::new();

        let ahora = Utc::now();
        let inicio = ahora + Duration::hours(1);
        let fin = inicio + Duration::hours(2);

        let mut reserva = Reserva::new("sala1".into(), "usuario1".into(), inicio, fin).unwrap();
        let id = reserva.id().to_string();

        repo.guardar(&reserva).await.unwrap();

        // Cancelar la reserva
        reserva.cancelar();
        repo.actualizar(&reserva).await.unwrap();

        let obtenida = repo.obtener(&id).await.unwrap().unwrap();
        assert_eq!(obtenida.estado(), &EstadoReserva::Cancelada);
    }

    #[tokio::test]
    async fn test_eliminar_reserva() {
        let repo = InMemoryReservaRepository::new();

        let ahora = Utc::now();
        let inicio = ahora + Duration::hours(1);
        let fin = inicio + Duration::hours(2);

        let reserva = Reserva::new("sala1".into(), "usuario1".into(), inicio, fin).unwrap();
        let id = reserva.id().to_string();

        repo.guardar(&reserva).await.unwrap();

        assert_eq!(repo.count().await, 1);

        repo.eliminar(&id).await.unwrap();

        assert_eq!(repo.count().await, 0);
        assert!(repo.obtener(&id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_clear() {
        let repo = InMemoryReservaRepository::new();

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

        assert_eq!(repo.count().await, 2);

        repo.clear().await;

        assert_eq!(repo.count().await, 0);
    }
}
