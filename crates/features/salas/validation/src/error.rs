use thiserror::Error;

/// Errores de validación para las operaciones de salas
#[derive(Debug, Error, Clone)]
pub enum SalaValidationError {
    // Errores de validación de nombre
    #[error("El nombre no puede estar vacío")]
    NombreVacio,

    #[error("El nombre debe tener entre {min} y {max} caracteres (actual: {actual})")]
    NombreLongitudInvalida { min: usize, max: usize, actual: usize },

    #[error("El nombre solo puede contener letras, números y espacios")]
    NombreCaracteresInvalidos,

    // Errores de validación de capacidad
    #[error("La capacidad debe ser mayor que 0")]
    CapacidadCero,

    #[error("La capacidad debe estar entre {min} y {max} (actual: {actual})")]
    CapacidadFueraDeRango { min: u32, max: u32, actual: u32 },

    // Errores de validación de ID
    #[error("El ID no puede estar vacío")]
    IdVacio,

    #[error("El ID debe ser un UUID válido")]
    IdFormatoInvalido,
}

impl SalaValidationError {
    /// Convierte el error a un mensaje de usuario amigable
    pub fn mensaje_usuario(&self) -> String {
        match self {
            Self::NombreVacio => "Por favor, ingresa un nombre para la sala".to_string(),
            Self::NombreLongitudInvalida { min, max, actual } => {
                format!(
                    "El nombre debe tener entre {} y {} caracteres. Actualmente tiene {}",
                    min, max, actual
                )
            }
            Self::NombreCaracteresInvalidos => {
                "El nombre solo puede contener letras, números y espacios".to_string()
            }
            Self::CapacidadCero => "La capacidad debe ser al menos 1 persona".to_string(),
            Self::CapacidadFueraDeRango { min, max, actual } => {
                format!(
                    "La capacidad debe estar entre {} y {} personas. Valor actual: {}",
                    min, max, actual
                )
            }
            Self::IdVacio => "El ID de la sala no puede estar vacío".to_string(),
            Self::IdFormatoInvalido => "El ID de la sala no es válido".to_string(),
        }
    }
}
