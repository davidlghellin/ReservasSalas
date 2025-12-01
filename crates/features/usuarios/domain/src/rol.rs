use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Roles de usuario en el sistema
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]

pub enum Rol {
    /// Administrador del sistema - puede gestionar salas y usuarios
    Admin,
    /// Usuario normal - puede crear y gestionar sus propias reservas
    #[default]
    Usuario,
}

impl Rol {
    /// Verifica si el rol tiene permisos de administrador
    pub fn es_admin(&self) -> bool {
        matches!(self, Rol::Admin)
    }

    /// Convierte el rol a string para almacenamiento
    pub fn as_str(&self) -> &str {
        match self {
            Rol::Admin => "admin",
            Rol::Usuario => "usuario",
        }
    }

    /// Crea un Rol desde un string
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "admin" => Some(Rol::Admin),
            "usuario" => Some(Rol::Usuario),
            _ => None,
        }
    }
}
impl FromStr for Rol {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Rol::from_str_opt(s).ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_es_admin() {
        assert!(Rol::Admin.es_admin());
        assert!(!Rol::Usuario.es_admin());
    }

    #[test]
    fn test_as_str() {
        assert_eq!(Rol::Admin.as_str(), "admin");
        assert_eq!(Rol::Usuario.as_str(), "usuario");
    }

    #[test]
    fn test_from_str() {
        assert_eq!(Rol::from_str_opt("admin"), Some(Rol::Admin));
        assert_eq!(Rol::from_str_opt("ADMIN"), Some(Rol::Admin));
        assert_eq!(Rol::from_str_opt("usuario"), Some(Rol::Usuario));
        assert_eq!(Rol::from_str_opt("USUARIO"), Some(Rol::Usuario));
        assert_eq!(Rol::from_str_opt("invalid"), None);
    }

    #[test]
    fn test_default() {
        assert_eq!(Rol::default(), Rol::Usuario);
    }
}
