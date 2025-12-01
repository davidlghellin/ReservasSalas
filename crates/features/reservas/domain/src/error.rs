use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ReservaError {
    SalaIdVacio,
    UsuarioIdVacio,
    FechaInicioInvalida,
    FechaFinInvalida,
    FechaFinAnteriorAInicio,
    DuracionInvalida,
    NoEncontrada,
    ErrorRepositorio(String),
    Validacion(Vec<String>),
}

impl fmt::Display for ReservaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReservaError::SalaIdVacio => write!(f, "El ID de sala no puede estar vacío"),
            ReservaError::UsuarioIdVacio => write!(f, "El ID de usuario no puede estar vacío"),
            ReservaError::FechaInicioInvalida => {
                write!(f, "La fecha de inicio no puede ser en el pasado")
            }
            ReservaError::FechaFinInvalida => {
                write!(f, "La fecha de fin no puede ser en el pasado")
            }
            ReservaError::FechaFinAnteriorAInicio => {
                write!(f, "La fecha de fin debe ser posterior a la fecha de inicio")
            }
            ReservaError::DuracionInvalida => {
                write!(
                    f,
                    "La duración de la reserva debe ser entre 15 minutos y 8 horas"
                )
            }
            ReservaError::NoEncontrada => write!(f, "Reserva no encontrada"),
            ReservaError::ErrorRepositorio(msg) => write!(f, "Error en repositorio: {}", msg),
            ReservaError::Validacion(msgs) => {
                write!(f, "Errores de validación: {}", msgs.join("; "))
            }
        }
    }
}

impl std::error::Error for ReservaError {}
