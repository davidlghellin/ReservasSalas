use crate::error::UsuarioError;
use crate::rol::Rol;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Longitud mínima del nombre de usuario
pub const NOMBRE_MIN_LENGTH: usize = 2;
/// Longitud máxima del nombre de usuario
pub const NOMBRE_MAX_LENGTH: usize = 100;
/// Longitud mínima de la contraseña
pub const PASSWORD_MIN_LENGTH: usize = 8;

/// Entidad Usuario del dominio
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Usuario {
    /// Identificador único del usuario
    pub id: String,
    /// Nombre completo del usuario
    pub nombre: String,
    /// Email del usuario (único en el sistema)
    pub email: String,
    /// Hash de la contraseña (nunca se guarda en texto plano)
    pub password_hash: String,
    /// Rol del usuario en el sistema
    pub rol: Rol,
    /// Fecha de creación del usuario
    pub created_at: DateTime<Utc>,
    /// Fecha de última actualización
    pub updated_at: DateTime<Utc>,
    /// Si el usuario está activo en el sistema
    pub activo: bool,
}

impl Usuario {
    /// Crea un nuevo usuario con validaciones
    ///
    /// # Argumentos
    /// * `nombre` - Nombre completo del usuario
    /// * `email` - Email del usuario
    /// * `password_hash` - Hash de la contraseña (ya hasheada)
    /// * `rol` - Rol del usuario
    ///
    /// # Errores
    /// Retorna error si las validaciones fallan
    pub fn new(
        nombre: String,
        email: String,
        password_hash: String,
        rol: Rol,
    ) -> Result<Self, UsuarioError> {
        // Validar nombre
        validar_nombre(&nombre)?;

        // Validar email
        validar_email(&email)?;

        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            nombre,
            email,
            password_hash,
            rol,
            created_at: now,
            updated_at: now,
            activo: true,
        })
    }

    /// Crea un usuario con ID específico (para carga desde persistencia)
    #[allow(clippy::too_many_arguments)]
    pub fn with_id(
        id: String,
        nombre: String,
        email: String,
        password_hash: String,
        rol: Rol,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        activo: bool,
    ) -> Result<Self, UsuarioError> {
        // Validar nombre y email
        validar_nombre(&nombre)?;
        validar_email(&email)?;

        Ok(Self {
            id,
            nombre,
            email,
            password_hash,
            rol,
            created_at,
            updated_at,
            activo,
        })
    }

    /// Actualiza el nombre del usuario
    pub fn actualizar_nombre(&mut self, nuevo_nombre: String) -> Result<(), UsuarioError> {
        validar_nombre(&nuevo_nombre)?;
        self.nombre = nuevo_nombre;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Actualiza el email del usuario
    pub fn actualizar_email(&mut self, nuevo_email: String) -> Result<(), UsuarioError> {
        validar_email(&nuevo_email)?;
        self.email = nuevo_email;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Actualiza el hash de la contraseña
    pub fn actualizar_password(&mut self, nuevo_hash: String) {
        self.password_hash = nuevo_hash;
        self.updated_at = Utc::now();
    }

    /// Actualiza el rol del usuario
    pub fn actualizar_rol(&mut self, nuevo_rol: Rol) {
        self.rol = nuevo_rol;
        self.updated_at = Utc::now();
    }

    /// Desactiva el usuario
    pub fn desactivar(&mut self) {
        self.activo = false;
        self.updated_at = Utc::now();
    }

    /// Activa el usuario
    pub fn activar(&mut self) {
        self.activo = true;
        self.updated_at = Utc::now();
    }

    /// Verifica si el usuario es administrador
    pub fn es_admin(&self) -> bool {
        self.rol.es_admin()
    }

    /// Retorna la información del usuario sin el password hash
    pub fn sin_password(&self) -> UsuarioPublico {
        UsuarioPublico {
            id: self.id.clone(),
            nombre: self.nombre.clone(),
            email: self.email.clone(),
            rol: self.rol.clone(),
            created_at: self.created_at,
            activo: self.activo,
        }
    }
}

/// Información pública del usuario (sin contraseña)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsuarioPublico {
    pub id: String,
    pub nombre: String,
    pub email: String,
    pub rol: Rol,
    pub created_at: DateTime<Utc>,
    pub activo: bool,
}

/// Valida que el nombre cumpla con las reglas de negocio
pub fn validar_nombre(nombre: &str) -> Result<(), UsuarioError> {
    let nombre_trimmed = nombre.trim();

    if nombre_trimmed.is_empty() {
        return Err(UsuarioError::NombreVacio);
    }

    let len = nombre_trimmed.len();
    if !(NOMBRE_MIN_LENGTH..=NOMBRE_MAX_LENGTH).contains(&len) {
        return Err(UsuarioError::NombreLongitudInvalida {
            min: NOMBRE_MIN_LENGTH,
            max: NOMBRE_MAX_LENGTH,
            actual: len,
        });
    }

    Ok(())
}

/// Valida que el email tenga formato correcto
pub fn validar_email(email: &str) -> Result<(), UsuarioError> {
    let email_trimmed = email.trim();

    if email_trimmed.is_empty() {
        return Err(UsuarioError::EmailInvalido(email.to_string()));
    }

    // Validación básica de email: debe contener @ y un punto después del @
    if !email_trimmed.contains('@') {
        return Err(UsuarioError::EmailInvalido(email.to_string()));
    }

    let parts: Vec<&str> = email_trimmed.split('@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(UsuarioError::EmailInvalido(email.to_string()));
    }

    if !parts[1].contains('.') {
        return Err(UsuarioError::EmailInvalido(email.to_string()));
    }

    Ok(())
}

/// Valida que la contraseña cumpla con los requisitos mínimos
pub fn validar_password(password: &str) -> Result<(), UsuarioError> {
    let len = password.len();

    if len < PASSWORD_MIN_LENGTH {
        return Err(UsuarioError::ContrasenaDemasiadoCorta {
            min: PASSWORD_MIN_LENGTH,
            actual: len,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_usuario_valido() {
        let usuario = Usuario::new(
            "Juan Pérez".to_string(),
            "juan@example.com".to_string(),
            "hashed_password".to_string(),
            Rol::Usuario,
        );

        assert!(usuario.is_ok());
        let u = usuario.unwrap();
        assert_eq!(u.nombre, "Juan Pérez");
        assert_eq!(u.email, "juan@example.com");
        assert!(u.activo);
        assert!(!u.es_admin());
    }

    #[test]
    fn test_crear_usuario_admin() {
        let usuario = Usuario::new(
            "Admin".to_string(),
            "admin@example.com".to_string(),
            "hashed".to_string(),
            Rol::Admin,
        );

        assert!(usuario.is_ok());
        assert!(usuario.unwrap().es_admin());
    }

    #[test]
    fn test_validar_nombre_vacio() {
        let result = validar_nombre("");
        assert!(matches!(result, Err(UsuarioError::NombreVacio)));
    }

    #[test]
    fn test_validar_nombre_muy_corto() {
        let result = validar_nombre("A");
        assert!(matches!(
            result,
            Err(UsuarioError::NombreLongitudInvalida { .. })
        ));
    }

    #[test]
    fn test_validar_email_invalido() {
        assert!(validar_email("").is_err());
        assert!(validar_email("notanemail").is_err());
        assert!(validar_email("no@domain").is_err());
        assert!(validar_email("@domain.com").is_err());
    }

    #[test]
    fn test_validar_email_valido() {
        assert!(validar_email("user@example.com").is_ok());
        assert!(validar_email("test.user@domain.co.uk").is_ok());
    }

    #[test]
    fn test_validar_password_muy_corta() {
        let result = validar_password("123");
        assert!(matches!(
            result,
            Err(UsuarioError::ContrasenaDemasiadoCorta { .. })
        ));
    }

    #[test]
    fn test_validar_password_valida() {
        assert!(validar_password("password123").is_ok());
    }

    #[test]
    fn test_actualizar_nombre() {
        let mut usuario = Usuario::new(
            "Original".to_string(),
            "test@example.com".to_string(),
            "hash".to_string(),
            Rol::Usuario,
        )
        .unwrap();

        let result = usuario.actualizar_nombre("Nuevo Nombre".to_string());
        assert!(result.is_ok());
        assert_eq!(usuario.nombre, "Nuevo Nombre");
    }

    #[test]
    fn test_desactivar_usuario() {
        let mut usuario = Usuario::new(
            "Test".to_string(),
            "test@example.com".to_string(),
            "hash".to_string(),
            Rol::Usuario,
        )
        .unwrap();

        assert!(usuario.activo);
        usuario.desactivar();
        assert!(!usuario.activo);
    }

    #[test]
    fn test_sin_password() {
        let usuario = Usuario::new(
            "Test".to_string(),
            "test@example.com".to_string(),
            "secret_hash".to_string(),
            Rol::Usuario,
        )
        .unwrap();

        let publico = usuario.sin_password();
        assert_eq!(publico.nombre, "Test");
        assert_eq!(publico.email, "test@example.com");
    }
}
