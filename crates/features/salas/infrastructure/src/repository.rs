use async_trait::async_trait;
use salas_application::SalaRepository;
use salas_domain::{Sala, SalaError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct InMemorySalaRepository {
    store: Arc<RwLock<HashMap<String, Sala>>>,
}

impl InMemorySalaRepository {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemorySalaRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SalaRepository for InMemorySalaRepository {
    async fn guardar(&self, sala: &Sala) -> Result<(), SalaError> {
        let mut store = self
            .store
            .write()
            .map_err(|e| SalaError::ErrorRepositorio(format!("Error al escribir: {}", e)))?;
        store.insert(sala.id.clone(), sala.clone());
        Ok(())
    }

    async fn obtener(&self, id: &str) -> Result<Option<Sala>, SalaError> {
        let store = self
            .store
            .read()
            .map_err(|e| SalaError::ErrorRepositorio(format!("Error al leer: {}", e)))?;
        Ok(store.get(id).cloned())
    }

    async fn listar(&self) -> Result<Vec<Sala>, SalaError> {
        let store = self
            .store
            .read()
            .map_err(|e| SalaError::ErrorRepositorio(format!("Error al leer: {}", e)))?;
        Ok(store.values().cloned().collect())
    }

    async fn actualizar(&self, sala: &Sala) -> Result<(), SalaError> {
        let mut store = self
            .store
            .write()
            .map_err(|e| SalaError::ErrorRepositorio(format!("Error al escribir: {}", e)))?;
        store.insert(sala.id.clone(), sala.clone());
        Ok(())
    }
}
