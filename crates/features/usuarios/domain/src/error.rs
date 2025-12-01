use thiserror::Error;

/// Errores relacionados con el dominio de usuarios
#[derive(Debug, Error, Clone)]
pub enum UsuarioError {
    #[error("El email '{0}' no es válido")]
    EmailInvalido(String),

    #[error("El email '{0}' ya está registrado")]
    EmailDuplicado(String),

    #[error("El nombre no puede estar vacío")]
    NombreVacio,

    #[error("El nombre debe tener entre {min} y {max} caracteres, pero tiene {actual}")]
    NombreLongitudInvalida {
        min: usize,
        max: usize,
        actual: usize,
    },

    #[error("La contraseña debe tener al menos {min} caracteres, pero tiene {actual}")]
    ContrasenaDemasiadoCorta { min: usize, actual: usize },

    #[error("Usuario no encontrado con ID: {0}")]
    UsuarioNoEncontrado(String),

    #[error("Credenciales inválidas")]
    CredencialesInvalidas,

    #[error("No tienes permisos para realizar esta acción")]
    PermisosDenegados,

    #[error("Error en el repositorio: {0}")]
    ErrorRepositorio(String),

    #[error("Error de validación: {0}")]
    ValidacionError(String),
}

impl UsuarioError {
    /// Obtiene un mensaje amigable para mostrar al usuario
    pub fn mensaje_usuario(&self) -> String {
        match self {
            UsuarioError::EmailInvalido(email) => {
                format!(
                    "El email '{}' no es válido. Por favor, ingresa un email correcto.",
                    email
                )
            }
            UsuarioError::EmailDuplicado(email) => {
                format!(
                    "El email '{}' ya está registrado. Intenta con otro email o inicia sesión.",
                    email
                )
            }
            UsuarioError::NombreVacio => {
                "El nombre no puede estar vacío. Por favor, ingresa tu nombre.".to_string()
            }
            UsuarioError::NombreLongitudInvalida { min, max, actual } => {
                format!(
                    "El nombre debe tener entre {} y {} caracteres (actualmente tiene {}).",
                    min, max, actual
                )
            }
            UsuarioError::ContrasenaDemasiadoCorta { min, actual } => {
                format!(
                    "La contraseña debe tener al menos {} caracteres (actualmente tiene {}).",
                    min, actual
                )
            }
            UsuarioError::UsuarioNoEncontrado(id) => {
                format!("No se encontró el usuario con ID: {}", id)
            }
            UsuarioError::CredencialesInvalidas => {
                "Email o contraseña incorrectos. Por favor, verifica tus credenciales.".to_string()
            }
            UsuarioError::PermisosDenegados => {
                "No tienes permisos para realizar esta acción.".to_string()
            }
            UsuarioError::ErrorRepositorio(msg) => {
                format!("Error al acceder a los datos: {}", msg)
            }
            UsuarioError::ValidacionError(msg) => {
                format!("Error de validación: {}", msg)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mensaje_usuario_email_invalido() {
        let error = UsuarioError::EmailInvalido("test@".to_string());
        assert!(error.mensaje_usuario().contains("test@"));
        assert!(error.mensaje_usuario().contains("no es válido"));
    }

    #[test]
    fn test_mensaje_usuario_credenciales_invalidas() {
        let error = UsuarioError::CredencialesInvalidas;
        assert!(error.mensaje_usuario().contains("incorrectos"));
    }
}
