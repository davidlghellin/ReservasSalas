use axum::extract::Request;
use axum::http::{header::AUTHORIZATION, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::middleware::Next;
use usuarios_auth::jwt::JwtService;
use usuarios_domain::Rol;

/// Extrae los claims JWT del request para usar en los handlers
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub email: String,
    pub rol: Rol,
}

/// Middleware de autenticación que valida tokens JWT
pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, AuthError> {
    // Extraer el token del header Authorization
    let token = extract_token(&req)?;

    // Validar el token JWT
    let claims = JwtService::validate_token(&token)
        .map_err(|_| AuthError::InvalidToken)?;

    // Convertir el rol string a enum Rol
    let rol = match claims.rol.as_str() {
        "admin" => Rol::Admin,
        "usuario" => Rol::Usuario,
        _ => return Err(AuthError::InvalidRole),
    };

    // Crear AuthUser y agregarlo a las extensiones del request
    let auth_user = AuthUser {
        user_id: claims.sub,
        email: claims.email,
        rol,
    };

    req.extensions_mut().insert(auth_user);

    Ok(next.run(req).await)
}

/// Middleware que requiere rol de administrador
/// Incluye validación de token JWT
pub async fn admin_middleware(mut req: Request, next: Next) -> Result<Response, AuthError> {
    // Primero validar token (reutilizar lógica de auth_middleware)
    let token = extract_token(&req)?;
    let claims = JwtService::validate_token(&token)
        .map_err(|_| AuthError::InvalidToken)?;

    let rol = match claims.rol.as_str() {
        "admin" => Rol::Admin,
        "usuario" => Rol::Usuario,
        _ => return Err(AuthError::InvalidRole),
    };

    // Verificar que sea admin
    if rol != Rol::Admin {
        return Err(AuthError::Forbidden);
    }

    // Crear AuthUser y agregarlo a las extensiones
    let auth_user = AuthUser {
        user_id: claims.sub,
        email: claims.email,
        rol,
    };

    req.extensions_mut().insert(auth_user);

    Ok(next.run(req).await)
}

/// Extrae el token JWT del header Authorization
fn extract_token(req: &Request) -> Result<String, AuthError> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(AuthError::MissingToken)?;

    // El formato debe ser "Bearer <token>"
    if !auth_header.starts_with("Bearer ") {
        return Err(AuthError::InvalidFormat);
    }

    Ok(auth_header[7..].to_string())
}

/// Errors de autenticación
#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidFormat,
    InvalidToken,
    InvalidRole,
    Unauthorized,
    Forbidden,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingToken => (
                StatusCode::UNAUTHORIZED,
                "Token de autenticación requerido".to_string(),
            ),
            AuthError::InvalidFormat => (
                StatusCode::BAD_REQUEST,
                "Formato de token inválido. Use: Bearer <token>".to_string(),
            ),
            AuthError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                "Token inválido o expirado".to_string(),
            ),
            AuthError::InvalidRole => (
                StatusCode::FORBIDDEN,
                "Rol inválido en el token".to_string(),
            ),
            AuthError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "No autorizado".to_string(),
            ),
            AuthError::Forbidden => (
                StatusCode::FORBIDDEN,
                "Se requiere rol de administrador".to_string(),
            ),
        };

        (status, axum::Json(serde_json::json!({ "error": message }))).into_response()
    }
}

/// Extension trait para facilitar la extracción de AuthUser
pub trait RequestExt {
    fn auth_user(&self) -> Option<&AuthUser>;
}

impl RequestExt for Request {
    fn auth_user(&self) -> Option<&AuthUser> {
        self.extensions().get::<AuthUser>()
    }
}

