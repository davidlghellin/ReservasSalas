use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reservas_domain::{Reserva, ReservaError};

/// Port (interfaz) del repositorio de reservas
#[async_trait]
pub trait ReservaRepository: Send + Sync {
    /// Guarda una nueva reserva
    async fn guardar(&self, reserva: &Reserva) -> Result<(), ReservaError>;

    /// Obtiene una reserva por su ID
    async fn obtener(&self, id: &str) -> Result<Option<Reserva>, ReservaError>;

    /// Lista todas las reservas
    async fn listar(&self) -> Result<Vec<Reserva>, ReservaError>;

    /// Lista reservas de una sala específica
    async fn listar_por_sala(&self, sala_id: &str) -> Result<Vec<Reserva>, ReservaError>;

    /// Lista reservas de un usuario específico
    async fn listar_por_usuario(&self, usuario_id: &str) -> Result<Vec<Reserva>, ReservaError>;

    /// Lista reservas activas de una sala en un rango de fechas
    async fn listar_por_sala_y_rango(
        &self,
        sala_id: &str,
        inicio: DateTime<Utc>,
        fin: DateTime<Utc>,
    ) -> Result<Vec<Reserva>, ReservaError>;

    /// Actualiza una reserva existente
    async fn actualizar(&self, reserva: &Reserva) -> Result<(), ReservaError>;

    /// Elimina una reserva por su ID
    async fn eliminar(&self, id: &str) -> Result<(), ReservaError>;
}
