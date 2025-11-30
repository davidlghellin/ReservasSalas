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

/// Trait de extensión para facilitar la extracción de AuthUser desde Request
pub trait RequestAuthExt {
    fn require_auth_user(&self) -> Result<AuthUser, Status>;
}

impl<T> RequestAuthExt for Request<T> {
    fn require_auth_user(&self) -> Result<AuthUser, Status> {
        extract_auth_user(self)
    }
}
