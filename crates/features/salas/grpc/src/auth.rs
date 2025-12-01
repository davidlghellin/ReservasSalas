use tonic::{Request, Status};
use usuarios_auth::jwt::{Claims, JwtService};
use usuarios_domain::Rol;

/// Información del usuario autenticado extraída del JWT
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub email: String,
    pub rol: Rol,
}

impl From<Claims> for AuthUser {
    fn from(claims: Claims) -> Self {
        let rol = match claims.rol.as_str() {
            "admin" => Rol::Admin,
            _ => Rol::Usuario,
        };

        Self {
            user_id: claims.sub,
            email: claims.email,
            rol,
        }
    }
}

/// Extrae y valida el token JWT del metadata de gRPC
///
/// El token debe venir en el header "authorization" con formato "Bearer <token>"
#[allow(clippy::result_large_err)]
pub fn extract_auth_user<T>(request: &Request<T>) -> Result<AuthUser, Status> {
    // Obtener el valor del header authorization
    let token = request
        .metadata()
        .get("authorization")
        .ok_or_else(|| Status::unauthenticated("Token de autenticación requerido"))?
        .to_str()
        .map_err(|_| Status::unauthenticated("Token de autenticación inválido"))?;

    // Verificar formato "Bearer <token>"
    if !token.starts_with("Bearer ") {
        return Err(Status::unauthenticated(
            "Formato de token inválido. Use: Bearer <token>",
        ));
    }

    // Extraer el token
    let token = &token[7..];

    // Validar el token JWT
    let claims = JwtService::validate_token(token)
        .map_err(|_| Status::unauthenticated("Token inválido o expirado"))?;

    Ok(AuthUser::from(claims))
}

/// Extrae y valida que el usuario autenticado sea administrador
#[allow(clippy::result_large_err)]
pub fn extract_admin_user<T>(request: &Request<T>) -> Result<AuthUser, Status> {
    let auth_user = extract_auth_user(request)?;

    if auth_user.rol != Rol::Admin {
        return Err(Status::permission_denied(
            "Se requiere rol de administrador",
        ));
    }

    Ok(auth_user)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::metadata::MetadataValue;

    #[test]
    fn test_extract_auth_user_sin_token() {
        let request: Request<()> = Request::new(());
        let result = extract_auth_user(&request);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
    }

    #[test]
    fn test_extract_auth_user_token_valido() {
        let token =
            JwtService::generate_token("user-123", "test@example.com", Rol::Usuario).unwrap();

        let mut request: Request<()> = Request::new(());
        let metadata_value = MetadataValue::try_from(format!("Bearer {}", token)).unwrap();
        request
            .metadata_mut()
            .insert("authorization", metadata_value);

        let result = extract_auth_user(&request);

        assert!(result.is_ok());
        let auth_user = result.unwrap();
        assert_eq!(auth_user.user_id, "user-123");
        assert_eq!(auth_user.email, "test@example.com");
        assert_eq!(auth_user.rol, Rol::Usuario);
    }

    #[test]
    fn test_extract_admin_user_con_permisos() {
        let token =
            JwtService::generate_token("admin-123", "admin@example.com", Rol::Admin).unwrap();

        let mut request: Request<()> = Request::new(());
        let metadata_value = MetadataValue::try_from(format!("Bearer {}", token)).unwrap();
        request
            .metadata_mut()
            .insert("authorization", metadata_value);

        let result = extract_admin_user(&request);

        assert!(result.is_ok());
        let auth_user = result.unwrap();
        assert_eq!(auth_user.rol, Rol::Admin);
    }
}
