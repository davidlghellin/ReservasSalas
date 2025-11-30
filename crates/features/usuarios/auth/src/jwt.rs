use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use usuarios_domain::Rol;

/// Clave secreta para firmar JWT (EN PRODUCCIÓN: usar variable de entorno)
const JWT_SECRET: &str = "tu_clave_secreta_super_segura_cambiar_en_produccion";

/// Tiempo de expiración del token (24 horas)
const TOKEN_EXPIRATION_HOURS: i64 = 24;

/// Claims (datos) incluidos en el JWT
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (user_id)
    pub sub: String,
    /// Email del usuario
    pub email: String,
    /// Rol del usuario
    pub rol: String,
    /// Timestamp de expiración
    pub exp: i64,
    /// Timestamp de emisión
    pub iat: i64,
}

/// Servicio para generar y validar tokens JWT
pub struct JwtService;

impl JwtService {
    /// Genera un token JWT para un usuario
    ///
    /// # Argumentos
    /// * `user_id` - ID del usuario
    /// * `email` - Email del usuario
    /// * `rol` - Rol del usuario
    ///
    /// # Retorna
    /// String con el token JWT
    ///
    /// # Errores
    /// Retorna error si la generación del token falla
    pub fn generate_token(user_id: &str, email: &str, rol: Rol) -> Result<String, String> {
        let now = Utc::now();
        let exp = (now + Duration::hours(TOKEN_EXPIRATION_HOURS))
            .timestamp();

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            rol: rol.as_str().to_string(),
            exp,
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
        )
        .map_err(|e| format!("Error al generar token: {}", e))
    }

    /// Valida un token JWT y extrae los claims
    ///
    /// # Argumentos
    /// * `token` - Token JWT a validar
    ///
    /// # Retorna
    /// Claims del token si es válido
    ///
    /// # Errores
    /// Retorna error si el token es inválido o ha expirado
    pub fn validate_token(token: &str) -> Result<Claims, String> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| format!("Token inválido: {}", e))?;

        Ok(token_data.claims)
    }

    /// Extrae el user_id de un token sin validar expiración
    /// Útil para debugging, NO usar en producción para autenticación
    pub fn decode_without_validation(token: &str) -> Result<Claims, String> {
        let mut validation = Validation::default();
        validation.validate_exp = false;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
            &validation,
        )
        .map_err(|e| format!("Error al decodificar token: {}", e))?;

        Ok(token_data.claims)
    }

    /// Verifica si un usuario tiene rol de administrador según el token
    pub fn is_admin_token(claims: &Claims) -> bool {
        claims.rol == "admin"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token() {
        let token = JwtService::generate_token("user-123", "test@example.com", Rol::Usuario);

        assert!(token.is_ok());
        let token = token.unwrap();
        assert!(!token.is_empty());
        assert!(token.split('.').count() == 3); // JWT tiene 3 partes
    }

    #[test]
    fn test_validate_token_valido() {
        let token = JwtService::generate_token("user-456", "admin@example.com", Rol::Admin)
            .unwrap();

        let claims = JwtService::validate_token(&token);
        assert!(claims.is_ok());

        let claims = claims.unwrap();
        assert_eq!(claims.sub, "user-456");
        assert_eq!(claims.email, "admin@example.com");
        assert_eq!(claims.rol, "admin");
    }

    #[test]
    fn test_validate_token_invalido() {
        let result = JwtService::validate_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_admin_token() {
        let token_admin = JwtService::generate_token("1", "admin@test.com", Rol::Admin).unwrap();
        let claims_admin = JwtService::validate_token(&token_admin).unwrap();
        assert!(JwtService::is_admin_token(&claims_admin));

        let token_user = JwtService::generate_token("2", "user@test.com", Rol::Usuario).unwrap();
        let claims_user = JwtService::validate_token(&token_user).unwrap();
        assert!(!JwtService::is_admin_token(&claims_user));
    }

    #[test]
    fn test_token_contiene_claims_correctos() {
        let user_id = "test-uuid-123";
        let email = "test@example.com";
        let rol = Rol::Usuario;

        let token = JwtService::generate_token(user_id, email, rol).unwrap();
        let claims = JwtService::validate_token(&token).unwrap();

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.email, email);
        assert_eq!(claims.rol, "usuario");
        assert!(claims.exp > Utc::now().timestamp());
        assert!(claims.iat <= Utc::now().timestamp());
    }

    #[test]
    fn test_decode_without_validation() {
        let token = JwtService::generate_token("user-789", "test@test.com", Rol::Usuario).unwrap();

        let claims = JwtService::decode_without_validation(&token);
        assert!(claims.is_ok());
        assert_eq!(claims.unwrap().sub, "user-789");
    }
}
