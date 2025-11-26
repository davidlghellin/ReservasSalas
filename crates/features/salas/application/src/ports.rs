use async_trait::async_trait;
use salas_domain::{Sala, SalaError};

#[async_trait]
pub trait SalaService: Send + Sync {
    async fn crear_sala(&self, nombre: String, capacidad: u32) -> Result<Sala, SalaError>;
    async fn obtener_sala(&self, id: &str) -> Result<Option<Sala>, SalaError>;
    async fn listar_salas(&self) -> Result<Vec<Sala>, SalaError>;
    async fn activar_sala(&self, id: &str) -> Result<Sala, SalaError>;
    async fn desactivar_sala(&self, id: &str) -> Result<Sala, SalaError>;
}

#[async_trait]
pub trait SalaRepository: Send + Sync {
    async fn guardar(&self, sala: &Sala) -> Result<(), SalaError>;
    async fn obtener(&self, id: &str) -> Result<Option<Sala>, SalaError>;
    async fn listar(&self) -> Result<Vec<Sala>, SalaError>;
    async fn actualizar(&self, sala: &Sala) -> Result<(), SalaError>;
}
