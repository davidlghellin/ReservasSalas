use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum SalaError {
    NombreVacio,
    NombreDemasiadoLargo,
    CapacidadInvalida,
    NoEncontrada,
    ErrorRepositorio(String),
    Validacion(Vec<String>),
}

impl fmt::Display for SalaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SalaError::NombreVacio => write!(f, "El nombre no puede estar vacío"),
            SalaError::NombreDemasiadoLargo => {
                write!(f, "El nombre no puede exceder 100 caracteres")
            }
            SalaError::CapacidadInvalida => write!(f, "La capacidad debe ser entre 1 y 1000"),
            SalaError::NoEncontrada => write!(f, "Sala no encontrada"),
            SalaError::ErrorRepositorio(msg) => write!(f, "Error en repositorio: {}", msg),
            SalaError::Validacion(msgs) => write!(f, "Errores de validación: {}", msgs.join("; ")),
        }
    }
}

impl std::error::Error for SalaError {}

pub fn convertir_errores_validacion(e: validator::ValidationErrors) -> Vec<String> {
    use std::borrow::Cow;
    let mut errores = Vec::new();

    for (campo, errs) in e.field_errors() {
        for err in errs {
            // Si en el atributo del struct pusiste `message = "..."`, la tienes aquí
            let msg: Cow<'static, str> = err
                .message
                .clone()
                .unwrap_or_else(|| Cow::from(format!("Error en {} ({})", campo, err.code)));

            errores.push(msg.to_string());
        }
    }

    errores
}
