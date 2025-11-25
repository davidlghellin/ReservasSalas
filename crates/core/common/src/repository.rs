use async_trait::async_trait;

#[async_trait]
pub trait Repository<T> {
    async fn guardar(&self, entity: &T) -> Result<(), String>;
    async fn obtener(&self, id: &str) -> Result<Option<T>, String>;
    async fn listar(&self) -> Result<Vec<T>, String>;
    async fn actualizar(&self, entity: &T) -> Result<(), String>;
    async fn eliminar(&self, id: &str) -> Result<(), String>;
}