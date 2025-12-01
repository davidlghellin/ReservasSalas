use crate::repository::UsuarioRepository;
use async_trait::async_trait;
use std::sync::Arc;
use usuarios_domain::{Rol, UsuarioError, UsuarioPublico};

/// Port (interfaz) del servicio de gestión de usuarios
#[async_trait]
pub trait UsuarioService: Send + Sync {
    /// Obtiene un usuario por su ID
    async fn obtener_usuario(&self, id: String) -> Result<UsuarioPublico, UsuarioError>;

    /// Lista todos los usuarios
    async fn listar_usuarios(&self) -> Result<Vec<UsuarioPublico>, UsuarioError>;

    /// Actualiza el nombre de un usuario
    async fn actualizar_nombre(
        &self,
        user_id: String,
        nuevo_nombre: String,
    ) -> Result<UsuarioPublico, UsuarioError>;

    /// Actualiza el rol de un usuario (solo admins)
    async fn actualizar_rol(
        &self,
        admin_id: String,
        user_id: String,
        nuevo_rol: Rol,
    ) -> Result<UsuarioPublico, UsuarioError>;

    /// Desactiva un usuario (solo admins)
    async fn desactivar_usuario(
        &self,
        admin_id: String,
        user_id: String,
    ) -> Result<(), UsuarioError>;

    /// Activa un usuario (solo admins)
    async fn activar_usuario(&self, admin_id: String, user_id: String) -> Result<(), UsuarioError>;
}

/// Implementación del servicio de gestión de usuarios
pub struct UsuarioServiceImpl<R: UsuarioRepository> {
    repository: Arc<R>,
}

impl<R: UsuarioRepository> UsuarioServiceImpl<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// Verifica que un usuario sea admin
    async fn verificar_admin(&self, admin_id: &str) -> Result<(), UsuarioError> {
        let admin = self
            .repository
            .obtener(admin_id)
            .await?
            .ok_or(UsuarioError::UsuarioNoEncontrado(admin_id.to_string()))?;

        if !admin.es_admin() {
            return Err(UsuarioError::PermisosDenegados);
        }

        Ok(())
    }
}

#[async_trait]
impl<R: UsuarioRepository> UsuarioService for UsuarioServiceImpl<R> {
    async fn obtener_usuario(&self, id: String) -> Result<UsuarioPublico, UsuarioError> {
        let usuario = self
            .repository
            .obtener(&id)
            .await?
            .ok_or(UsuarioError::UsuarioNoEncontrado(id))?;

        Ok(usuario.sin_password())
    }

    async fn listar_usuarios(&self) -> Result<Vec<UsuarioPublico>, UsuarioError> {
        let usuarios = self.repository.listar().await?;
        Ok(usuarios.iter().map(|u| u.sin_password()).collect())
    }

    async fn actualizar_nombre(
        &self,
        user_id: String,
        nuevo_nombre: String,
    ) -> Result<UsuarioPublico, UsuarioError> {
        let mut usuario = self
            .repository
            .obtener(&user_id)
            .await?
            .ok_or(UsuarioError::UsuarioNoEncontrado(user_id))?;

        usuario.actualizar_nombre(nuevo_nombre)?;
        self.repository.actualizar(&usuario).await?;

        Ok(usuario.sin_password())
    }

    async fn actualizar_rol(
        &self,
        admin_id: String,
        user_id: String,
        nuevo_rol: Rol,
    ) -> Result<UsuarioPublico, UsuarioError> {
        // Verificar que quien llama sea admin
        self.verificar_admin(&admin_id).await?;

        // Obtener usuario a actualizar
        let mut usuario = self
            .repository
            .obtener(&user_id)
            .await?
            .ok_or(UsuarioError::UsuarioNoEncontrado(user_id))?;

        // Actualizar rol
        usuario.actualizar_rol(nuevo_rol);
        self.repository.actualizar(&usuario).await?;

        Ok(usuario.sin_password())
    }

    async fn desactivar_usuario(
        &self,
        admin_id: String,
        user_id: String,
    ) -> Result<(), UsuarioError> {
        // Verificar que quien llama sea admin
        self.verificar_admin(&admin_id).await?;

        // No permitir que un admin se desactive a sí mismo
        if admin_id == user_id {
            return Err(UsuarioError::ValidacionError(
                "No puedes desactivarte a ti mismo".to_string(),
            ));
        }

        // Obtener y desactivar usuario
        let mut usuario = self
            .repository
            .obtener(&user_id)
            .await?
            .ok_or(UsuarioError::UsuarioNoEncontrado(user_id))?;

        usuario.desactivar();
        self.repository.actualizar(&usuario).await?;

        Ok(())
    }

    async fn activar_usuario(&self, admin_id: String, user_id: String) -> Result<(), UsuarioError> {
        // Verificar que quien llama sea admin
        self.verificar_admin(&admin_id).await?;

        // Obtener y activar usuario
        let mut usuario = self
            .repository
            .obtener(&user_id)
            .await?
            .ok_or(UsuarioError::UsuarioNoEncontrado(user_id))?;

        usuario.activar();
        self.repository.actualizar(&usuario).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::RwLock;
    use usuarios_auth::PasswordService;
    use usuarios_domain::Usuario;

    // Mock repository
    struct MockUsuarioRepository {
        usuarios: Arc<RwLock<HashMap<String, Usuario>>>,
    }

    impl MockUsuarioRepository {
        fn new() -> Self {
            Self {
                usuarios: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl UsuarioRepository for MockUsuarioRepository {
        async fn guardar(&self, usuario: &Usuario) -> Result<(), UsuarioError> {
            let mut usuarios = self.usuarios.write().await;
            usuarios.insert(usuario.id.clone(), usuario.clone());
            Ok(())
        }

        async fn obtener(&self, id: &str) -> Result<Option<Usuario>, UsuarioError> {
            let usuarios = self.usuarios.read().await;
            Ok(usuarios.get(id).cloned())
        }

        async fn obtener_por_email(&self, email: &str) -> Result<Option<Usuario>, UsuarioError> {
            let usuarios = self.usuarios.read().await;
            Ok(usuarios.values().find(|u| u.email == email).cloned())
        }

        async fn listar(&self) -> Result<Vec<Usuario>, UsuarioError> {
            let usuarios = self.usuarios.read().await;
            Ok(usuarios.values().cloned().collect())
        }

        async fn actualizar(&self, usuario: &Usuario) -> Result<(), UsuarioError> {
            let mut usuarios = self.usuarios.write().await;
            usuarios.insert(usuario.id.clone(), usuario.clone());
            Ok(())
        }

        async fn eliminar(&self, id: &str) -> Result<(), UsuarioError> {
            let mut usuarios = self.usuarios.write().await;
            usuarios.remove(id);
            Ok(())
        }

        async fn existe_email(&self, email: &str) -> Result<bool, UsuarioError> {
            let usuarios = self.usuarios.read().await;
            Ok(usuarios.values().any(|u| u.email == email))
        }
    }

    async fn crear_usuario_test(
        repo: &Arc<MockUsuarioRepository>,
        nombre: &str,
        email: &str,
        rol: Rol,
    ) -> Usuario {
        let hash = PasswordService::hash_password("password123").unwrap();
        let usuario = Usuario::new(nombre.to_string(), email.to_string(), hash, rol).unwrap();
        repo.guardar(&usuario).await.unwrap();
        usuario
    }

    #[tokio::test]
    async fn test_listar_usuarios() {
        let repo = Arc::new(MockUsuarioRepository::new());
        let service = UsuarioServiceImpl::new(repo.clone());

        // Crear usuarios
        crear_usuario_test(&repo, "User 1", "user1@test.com", Rol::Usuario).await;
        crear_usuario_test(&repo, "User 2", "user2@test.com", Rol::Usuario).await;

        let usuarios = service.listar_usuarios().await.unwrap();
        assert_eq!(usuarios.len(), 2);
    }

    #[tokio::test]
    async fn test_actualizar_rol_como_admin() {
        let repo = Arc::new(MockUsuarioRepository::new());
        let service = UsuarioServiceImpl::new(repo.clone());

        let admin = crear_usuario_test(&repo, "Admin", "admin@test.com", Rol::Admin).await;
        let usuario = crear_usuario_test(&repo, "User", "user@test.com", Rol::Usuario).await;

        // Admin actualiza rol de usuario
        let result = service
            .actualizar_rol(admin.id, usuario.id.clone(), Rol::Admin)
            .await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.rol, Rol::Admin);
    }

    #[tokio::test]
    async fn test_actualizar_rol_sin_permisos() {
        let repo = Arc::new(MockUsuarioRepository::new());
        let service = UsuarioServiceImpl::new(repo.clone());

        let user1 = crear_usuario_test(&repo, "User 1", "user1@test.com", Rol::Usuario).await;
        let user2 = crear_usuario_test(&repo, "User 2", "user2@test.com", Rol::Usuario).await;

        // Usuario normal intenta actualizar rol
        let result = service.actualizar_rol(user1.id, user2.id, Rol::Admin).await;

        assert!(matches!(result, Err(UsuarioError::PermisosDenegados)));
    }

    #[tokio::test]
    async fn test_desactivar_usuario() {
        let repo = Arc::new(MockUsuarioRepository::new());
        let service = UsuarioServiceImpl::new(repo.clone());

        let admin = crear_usuario_test(&repo, "Admin", "admin@test.com", Rol::Admin).await;
        let usuario = crear_usuario_test(&repo, "User", "user@test.com", Rol::Usuario).await;

        // Desactivar usuario
        let result = service
            .desactivar_usuario(admin.id, usuario.id.clone())
            .await;

        assert!(result.is_ok());

        // Verificar que está desactivado
        let user_updated = repo.obtener(&usuario.id).await.unwrap().unwrap();
        assert!(!user_updated.activo);
    }

    #[tokio::test]
    async fn test_admin_no_puede_desactivarse_a_si_mismo() {
        let repo = Arc::new(MockUsuarioRepository::new());
        let service = UsuarioServiceImpl::new(repo.clone());

        let admin = crear_usuario_test(&repo, "Admin", "admin@test.com", Rol::Admin).await;

        let result = service
            .desactivar_usuario(admin.id.clone(), admin.id.clone())
            .await;

        assert!(matches!(result, Err(UsuarioError::ValidacionError(_))));
    }
}
