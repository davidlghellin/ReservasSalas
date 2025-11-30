use crate::error::SalaValidationError;
use salas_grpc::proto::{
    ActivarSalaRequest, CrearSalaRequest, DesactivarSalaRequest, ObtenerSalaRequest,
};

// Constantes de validación
pub const NOMBRE_MIN_LENGTH: usize = 3;
pub const NOMBRE_MAX_LENGTH: usize = 100;
pub const CAPACIDAD_MIN: u32 = 1;
pub const CAPACIDAD_MAX: u32 = 500;

/// Trait para validar requests de sala
pub trait ValidarSala {
    /// Valida el request y retorna un error si no es válido
    fn validar(&self) -> Result<(), SalaValidationError>;
}

impl ValidarSala for CrearSalaRequest {
    fn validar(&self) -> Result<(), SalaValidationError> {
        // Validar nombre
        validar_nombre(&self.nombre)?;

        // Validar capacidad
        validar_capacidad(self.capacidad)?;

        Ok(())
    }
}

impl ValidarSala for ObtenerSalaRequest {
    fn validar(&self) -> Result<(), SalaValidationError> {
        validar_id(&self.id)
    }
}

impl ValidarSala for ActivarSalaRequest {
    fn validar(&self) -> Result<(), SalaValidationError> {
        validar_id(&self.id)
    }
}

impl ValidarSala for DesactivarSalaRequest {
    fn validar(&self) -> Result<(), SalaValidationError> {
        validar_id(&self.id)
    }
}

// -------- Funciones de validación reutilizables --------

/// Valida el nombre de la sala
///
/// # Reglas
/// - No puede estar vacío (después de trim)
/// - Debe tener entre 3 y 100 caracteres
/// - Solo puede contener letras, números y espacios
pub fn validar_nombre(nombre: &str) -> Result<(), SalaValidationError> {
    // No puede estar vacío
    let nombre_trimmed = nombre.trim();
    if nombre_trimmed.is_empty() {
        return Err(SalaValidationError::NombreVacio);
    }

    // Validar longitud
    let len = nombre_trimmed.len();
    if len < NOMBRE_MIN_LENGTH || len > NOMBRE_MAX_LENGTH {
        return Err(SalaValidationError::NombreLongitudInvalida {
            min: NOMBRE_MIN_LENGTH,
            max: NOMBRE_MAX_LENGTH,
            actual: len,
        });
    }

    // Validar caracteres (solo alfanuméricos y espacios)
    if !nombre_trimmed
        .chars()
        .all(|c| c.is_alphanumeric() || c.is_whitespace())
    {
        return Err(SalaValidationError::NombreCaracteresInvalidos);
    }

    Ok(())
}

/// Valida la capacidad de la sala
///
/// # Reglas
/// - Debe ser mayor que 0
/// - Debe estar entre 1 y 500 personas
pub fn validar_capacidad(capacidad: u32) -> Result<(), SalaValidationError> {
    if capacidad == 0 {
        return Err(SalaValidationError::CapacidadCero);
    }

    if capacidad < CAPACIDAD_MIN || capacidad > CAPACIDAD_MAX {
        return Err(SalaValidationError::CapacidadFueraDeRango {
            min: CAPACIDAD_MIN,
            max: CAPACIDAD_MAX,
            actual: capacidad,
        });
    }

    Ok(())
}

/// Valida el ID de la sala (debe ser UUID válido)
///
/// # Reglas
/// - No puede estar vacío
/// - Debe ser un UUID válido (formato estándar)
pub fn validar_id(id: &str) -> Result<(), SalaValidationError> {
    if id.trim().is_empty() {
        return Err(SalaValidationError::IdVacio);
    }

    // Validar que sea un UUID válido
    if uuid::Uuid::parse_str(id).is_err() {
        return Err(SalaValidationError::IdFormatoInvalido);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validar_nombre_valido() {
        assert!(validar_nombre("Sala 101").is_ok());
        assert!(validar_nombre("Sala Principal").is_ok());
        assert!(validar_nombre("Sala de Reuniones 2024").is_ok());
    }

    #[test]
    fn test_validar_nombre_vacio() {
        assert!(matches!(
            validar_nombre(""),
            Err(SalaValidationError::NombreVacio)
        ));
        assert!(matches!(
            validar_nombre("   "),
            Err(SalaValidationError::NombreVacio)
        ));
    }

    #[test]
    fn test_validar_nombre_muy_corto() {
        assert!(matches!(
            validar_nombre("AB"),
            Err(SalaValidationError::NombreLongitudInvalida { .. })
        ));
    }

    #[test]
    fn test_validar_nombre_muy_largo() {
        let nombre_largo = "A".repeat(101);
        assert!(matches!(
            validar_nombre(&nombre_largo),
            Err(SalaValidationError::NombreLongitudInvalida { .. })
        ));
    }

    #[test]
    fn test_validar_nombre_caracteres_invalidos() {
        assert!(matches!(
            validar_nombre("Sala@#$%"),
            Err(SalaValidationError::NombreCaracteresInvalidos)
        ));
    }

    #[test]
    fn test_validar_capacidad_valida() {
        assert!(validar_capacidad(1).is_ok());
        assert!(validar_capacidad(50).is_ok());
        assert!(validar_capacidad(500).is_ok());
    }

    #[test]
    fn test_validar_capacidad_cero() {
        assert!(matches!(
            validar_capacidad(0),
            Err(SalaValidationError::CapacidadCero)
        ));
    }

    #[test]
    fn test_validar_capacidad_fuera_de_rango() {
        assert!(matches!(
            validar_capacidad(501),
            Err(SalaValidationError::CapacidadFueraDeRango { .. })
        ));
    }

    #[test]
    fn test_validar_id_valido() {
        let uuid_valido = "550e8400-e29b-41d4-a716-446655440000";
        assert!(validar_id(uuid_valido).is_ok());
    }

    #[test]
    fn test_validar_id_vacio() {
        assert!(matches!(
            validar_id(""),
            Err(SalaValidationError::IdVacio)
        ));
    }

    #[test]
    fn test_validar_id_formato_invalido() {
        assert!(matches!(
            validar_id("no-es-un-uuid"),
            Err(SalaValidationError::IdFormatoInvalido)
        ));
    }

    #[test]
    fn test_crear_sala_request_valido() {
        let request = CrearSalaRequest {
            nombre: "Sala 101".to_string(),
            capacidad: 50,
        };
        assert!(request.validar().is_ok());
    }

    #[test]
    fn test_crear_sala_request_nombre_invalido() {
        let request = CrearSalaRequest {
            nombre: "AB".to_string(), // Muy corto
            capacidad: 50,
        };
        assert!(request.validar().is_err());
    }

    #[test]
    fn test_crear_sala_request_capacidad_invalida() {
        let request = CrearSalaRequest {
            nombre: "Sala 101".to_string(),
            capacidad: 0, // Inválido
        };
        assert!(request.validar().is_err());
    }
}
