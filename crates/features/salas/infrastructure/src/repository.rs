use async_trait::async_trait;
use salas_application::SalaRepository;
use salas_domain::Sala;
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

#[async_trait]
impl SalaRepository for InMemorySalaRepository {
    async fn guardar(&self, sala: &Sala) -> Result<(), String> {
        let mut store = self.store.write().unwrap();
        store.insert(sala.id.clone(), sala.clone());
        Ok(())
    }

    async fn obtener(&self, id: &str) -> Result<Option<Sala>, String> {
        let store = self.store.read().unwrap();
        Ok(store.get(id).cloned())
    }

    async fn listar(&self) -> Result<Vec<Sala>, String> {
        let store = self.store.read().unwrap();
        Ok(store.values().cloned().collect())
    }

    async fn actualizar(&self, sala: &Sala) -> Result<(), String> {
        let mut store = self.store.write().unwrap();
        store.insert(sala.id.clone(), sala.clone());
        Ok(())
    }
}