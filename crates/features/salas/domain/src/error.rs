use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum SalaError {
    NombreVacio,
    NombreDemasiadoLargo,
    CapacidadInvalida,
    NoEncontrada,
    ErrorRepositorio(String),
}

impl fmt::Display for SalaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SalaError::NombreVacio => write!(f, "El nombre no puede estar vacío"),
            SalaError::NombreDemasiadoLargo => write!(f, "El nombre no puede exceder 100 caracteres"),
            SalaError::CapacidadInvalida => write!(f, "La capacidad debe ser entre 1 y 1000"),
            SalaError::NoEncontrada => write!(f, "Sala no encontrada"),
            SalaError::ErrorRepositorio(msg) => write!(f, "Error en repositorio: {}", msg),
        }
    }
}

impl std::error::Error for SalaError {}
