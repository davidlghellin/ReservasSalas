use async_trait::async_trait;
use usuarios_domain::{Usuario, UsuarioError};

/// Port (interfaz) del repositorio de usuarios
#[async_trait]
pub trait UsuarioRepository: Send + Sync {
    /// Guarda un nuevo usuario
    async fn guardar(&self, usuario: &Usuario) -> Result<(), UsuarioError>;

    /// Obtiene un usuario por su ID
    async fn obtener(&self, id: &str) -> Result<Option<Usuario>, UsuarioError>;

    /// Obtiene un usuario por su email
    async fn obtener_por_email(&self, email: &str) -> Result<Option<Usuario>, UsuarioError>;

    /// Lista todos los usuarios
    async fn listar(&self) -> Result<Vec<Usuario>, UsuarioError>;

    /// Actualiza un usuario existente
    async fn actualizar(&self, usuario: &Usuario) -> Result<(), UsuarioError>;

    /// Elimina un usuario por su ID
    async fn eliminar(&self, id: &str) -> Result<(), UsuarioError>;

    /// Verifica si existe un usuario con el email dado
    async fn existe_email(&self, email: &str) -> Result<bool, UsuarioError>;
}
