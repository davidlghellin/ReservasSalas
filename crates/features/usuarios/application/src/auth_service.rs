use crate::repository::UsuarioRepository;
use async_trait::async_trait;
use std::sync::Arc;
use usuarios_auth::{JwtService, PasswordService};
use usuarios_domain::{validar_password, Rol, Usuario, UsuarioError, UsuarioPublico};

/// Respuesta del login
#[derive(Debug, Clone)]
pub struct LoginResponse {
    pub token: String,
    pub usuario: UsuarioPublico,
}

/// Respuesta del registro
#[derive(Debug, Clone)]
pub struct RegisterResponse {
    pub token: String,
    pub usuario: UsuarioPublico,
}

/// Port (interfaz) del servicio de autenticación
#[async_trait]
pub trait AuthService: Send + Sync {
    /// Registra un nuevo usuario
    async fn register(
        &self,
        nombre: String,
        email: String,
        password: String,
        rol: Option<Rol>,
    ) -> Result<RegisterResponse, UsuarioError>;

    /// Autentica un usuario y genera un token JWT
    async fn login(&self, email: String, password: String) -> Result<LoginResponse, UsuarioError>;

    /// Valida un token JWT y retorna el usuario
    async fn validate_token(&self, token: String) -> Result<UsuarioPublico, UsuarioError>;

    /// Cambia la contraseña de un usuario
    async fn change_password(
        &self,
        user_id: String,
        old_password: String,
        new_password: String,
    ) -> Result<(), UsuarioError>;
}

/// Implementación del servicio de autenticación
pub struct AuthServiceImpl<R: UsuarioRepository> {
    repository: Arc<R>,
}

impl<R: UsuarioRepository> AuthServiceImpl<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: UsuarioRepository> AuthService for AuthServiceImpl<R> {
    async fn register(
        &self,
        nombre: String,
        email: String,
        password: String,
        rol: Option<Rol>,
    ) -> Result<RegisterResponse, UsuarioError> {
        // Validar contraseña
        validar_password(&password)?;

        // Verificar que el email no esté registrado
        if self.repository.existe_email(&email).await? {
            return Err(UsuarioError::EmailDuplicado(email));
        }

        // Hashear contraseña
        let password_hash = PasswordService::hash_password(&password)
            .map_err(|e| UsuarioError::ErrorRepositorio(e))?;

        // Crear usuario
        let rol = rol.unwrap_or(Rol::Usuario);
        let usuario = Usuario::new(nombre, email.clone(), password_hash, rol)?;

        // Guardar en repositorio
        self.repository.guardar(&usuario).await?;

        // Generar token JWT
        let token = JwtService::generate_token(&usuario.id, &usuario.email, usuario.rol.clone())
            .map_err(|e| UsuarioError::ErrorRepositorio(e))?;

        Ok(RegisterResponse {
            token,
            usuario: usuario.sin_password(),
        })
    }

    async fn login(&self, email: String, password: String) -> Result<LoginResponse, UsuarioError> {
        // Buscar usuario por email
        let usuario = self
            .repository
            .obtener_por_email(&email)
            .await?
            .ok_or(UsuarioError::CredencialesInvalidas)?;

        // Verificar que el usuario esté activo
        if !usuario.activo {
            return Err(UsuarioError::CredencialesInvalidas);
        }

        // Verificar contraseña
        let password_valida = PasswordService::verify_password(&password, &usuario.password_hash)
            .map_err(|e| UsuarioError::ErrorRepositorio(e))?;

        if !password_valida {
            return Err(UsuarioError::CredencialesInvalidas);
        }

        // Generar token JWT
        let token = JwtService::generate_token(&usuario.id, &usuario.email, usuario.rol.clone())
            .map_err(|e| UsuarioError::ErrorRepositorio(e))?;

        Ok(LoginResponse {
            token,
            usuario: usuario.sin_password(),
        })
    }

    async fn validate_token(&self, token: String) -> Result<UsuarioPublico, UsuarioError> {
        // Validar token JWT
        let claims = JwtService::validate_token(&token)
            .map_err(|_| UsuarioError::CredencialesInvalidas)?;

        // Obtener usuario del repositorio
        let usuario = self
            .repository
            .obtener(&claims.sub)
            .await?
            .ok_or(UsuarioError::UsuarioNoEncontrado(claims.sub))?;

        // Verificar que el usuario esté activo
        if !usuario.activo {
            return Err(UsuarioError::CredencialesInvalidas);
        }

        Ok(usuario.sin_password())
    }

    async fn change_password(
        &self,
        user_id: String,
        old_password: String,
        new_password: String,
    ) -> Result<(), UsuarioError> {
        // Validar nueva contraseña
        validar_password(&new_password)?;

        // Obtener usuario
        let mut usuario = self
            .repository
            .obtener(&user_id)
            .await?
            .ok_or(UsuarioError::UsuarioNoEncontrado(user_id))?;

        // Verificar contraseña actual
        let password_valida =
            PasswordService::verify_password(&old_password, &usuario.password_hash)
                .map_err(|e| UsuarioError::ErrorRepositorio(e))?;

        if !password_valida {
            return Err(UsuarioError::CredencialesInvalidas);
        }

        // Hashear nueva contraseña
        let new_password_hash = PasswordService::hash_password(&new_password)
            .map_err(|e| UsuarioError::ErrorRepositorio(e))?;

        // Actualizar usuario
        usuario.actualizar_password(new_password_hash);
        self.repository.actualizar(&usuario).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    // Mock repository para tests
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

    #[tokio::test]
    async fn test_register_usuario() {
        let repo = Arc::new(MockUsuarioRepository::new());
        let service = AuthServiceImpl::new(repo.clone());

        let result = service
            .register(
                "Test User".to_string(),
                "test@example.com".to_string(),
                "password123".to_string(),
                None,
            )
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.token.is_empty());
        assert_eq!(response.usuario.email, "test@example.com");
        assert_eq!(response.usuario.nombre, "Test User");
    }

    #[tokio::test]
    async fn test_register_email_duplicado() {
        let repo = Arc::new(MockUsuarioRepository::new());
        let service = AuthServiceImpl::new(repo.clone());

        // Registrar primer usuario
        service
            .register(
                "User 1".to_string(),
                "duplicate@example.com".to_string(),
                "password123".to_string(),
                None,
            )
            .await
            .unwrap();

        // Intentar registrar con mismo email
        let result = service
            .register(
                "User 2".to_string(),
                "duplicate@example.com".to_string(),
                "password456".to_string(),
                None,
            )
            .await;

        assert!(matches!(result, Err(UsuarioError::EmailDuplicado(_))));
    }

    #[tokio::test]
    async fn test_login_exitoso() {
        let repo = Arc::new(MockUsuarioRepository::new());
        let service = AuthServiceImpl::new(repo.clone());

        // Registrar usuario
        service
            .register(
                "Login User".to_string(),
                "login@example.com".to_string(),
                "mypassword".to_string(),
                None,
            )
            .await
            .unwrap();

        // Hacer login
        let result = service
            .login("login@example.com".to_string(), "mypassword".to_string())
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.token.is_empty());
        assert_eq!(response.usuario.email, "login@example.com");
    }

    #[tokio::test]
    async fn test_login_password_incorrecta() {
        let repo = Arc::new(MockUsuarioRepository::new());
        let service = AuthServiceImpl::new(repo.clone());

        // Registrar usuario
        service
            .register(
                "User".to_string(),
                "user@example.com".to_string(),
                "correctpassword".to_string(),
                None,
            )
            .await
            .unwrap();

        // Intentar login con password incorrecta
        let result = service
            .login("user@example.com".to_string(), "wrongpassword".to_string())
            .await;

        assert!(matches!(result, Err(UsuarioError::CredencialesInvalidas)));
    }

    #[tokio::test]
    async fn test_validate_token() {
        let repo = Arc::new(MockUsuarioRepository::new());
        let service = AuthServiceImpl::new(repo.clone());

        // Registrar y obtener token
        let register_response = service
            .register(
                "Token User".to_string(),
                "token@example.com".to_string(),
                "password123".to_string(),
                None,
            )
            .await
            .unwrap();

        // Validar token
        let result = service.validate_token(register_response.token).await;

        assert!(result.is_ok());
        let usuario = result.unwrap();
        assert_eq!(usuario.email, "token@example.com");
    }

    #[tokio::test]
    async fn test_change_password() {
        let repo = Arc::new(MockUsuarioRepository::new());
        let service = AuthServiceImpl::new(repo.clone());

        // Registrar usuario
        let register_response = service
            .register(
                "Change Pass User".to_string(),
                "change@example.com".to_string(),
                "oldpassword".to_string(),
                None,
            )
            .await
            .unwrap();

        // Cambiar contraseña
        let result = service
            .change_password(
                register_response.usuario.id.clone(),
                "oldpassword".to_string(),
                "newpassword123".to_string(),
            )
            .await;

        assert!(result.is_ok());

        // Verificar que el login con la nueva contraseña funciona
        let login_result = service
            .login("change@example.com".to_string(), "newpassword123".to_string())
            .await;

        assert!(login_result.is_ok());

        // Verificar que el login con la vieja contraseña falla
        let old_login_result = service
            .login("change@example.com".to_string(), "oldpassword".to_string())
            .await;

        assert!(matches!(
            old_login_result,
            Err(UsuarioError::CredencialesInvalidas)
        ));
    }
}
