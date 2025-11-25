use async_trait::async_trait;
use salas_domain::Sala;
use uuid::Uuid;

use crate::ports::{SalaRepository, SalaService};

pub struct SalaServiceImpl<R: SalaRepository> {
    repository: R,
}

impl<R: SalaRepository> SalaServiceImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: SalaRepository> SalaService for SalaServiceImpl<R> {
    async fn crear_sala(&self, nombre: String, capacidad: u32) -> Result<Sala, String> {
        let id = Uuid::new_v4().to_string();
        let sala = Sala::new(id, nombre, capacidad)?;
        self.repository.guardar(&sala).await?;
        Ok(sala)
    }

    async fn obtener_sala(&self, id: &str) -> Result<Option<Sala>, String> {
        self.repository.obtener(id).await
    }

    async fn listar_salas(&self) -> Result<Vec<Sala>, String> {
        self.repository.listar().await
    }

    async fn activar_sala(&self, id: &str) -> Result<Sala, String> {
        let mut sala = self
            .repository
            .obtener(id)
            .await?
            .ok_or_else(|| "Sala no encontrada".to_string())?;
        sala.activar();
        self.repository.actualizar(&sala).await?;
        Ok(sala)
    }

    async fn desactivar_sala(&self, id: &str) -> Result<Sala, String> {
        let mut sala = self
            .repository
            .obtener(id)
            .await?
            .ok_or_else(|| "Sala no encontrada".to_string())?;
        sala.desactivar();
        self.repository.actualizar(&sala).await?;
        Ok(sala)
    }
}
