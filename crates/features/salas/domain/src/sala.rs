use crate::error::SalaError;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct Sala {
    pub id: String,
    #[validate(length(min = 1, max = 100))]
    pub nombre: String,
    #[validate(range(min = 1, max = 1000))]
    pub capacidad: u32,
    pub activa: bool,
}

impl Sala {
    pub fn new(id: String, nombre: String, capacidad: u32) -> Result<Self, SalaError> {
        let nombre_trim = nombre.trim().to_string();
        if nombre_trim.is_empty() {
            return Err(SalaError::NombreVacio);
        }

        let sala = Self {
            id,
            nombre: nombre_trim.clone(),
            capacidad,
            activa: true,
        };

        if let Err(e) = sala.validate() {
            // mapear errores de campo a tus variantes
            if e.field_errors().contains_key("nombre") {
                return Err(SalaError::NombreDemasiadoLargo);
            }
            if e.field_errors().contains_key("capacidad") {
                return Err(SalaError::CapacidadInvalida);
            }
            // fallback: mapear a un error genérico si tu enum lo permite
            return Err(SalaError::CapacidadInvalida);
        }

        Ok(sala)
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn nombre(&self) -> &str {
        &self.nombre
    }

    pub fn capacidad(&self) -> u32 {
        self.capacidad
    }

    pub fn esta_activa(&self) -> bool {
        self.activa
    }

    pub fn activar(&mut self) {
        self.activa = true;
    }

    pub fn desactivar(&mut self) {
        self.activa = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crear_sala_valida() {
        let sala = Sala::new("123".to_string(), "Sala de Conferencias".to_string(), 10);

        assert!(sala.is_ok());
        let sala = sala.unwrap();
        assert_eq!(sala.id(), "123");
        assert_eq!(sala.nombre(), "Sala de Conferencias");
        assert_eq!(sala.capacidad(), 10);
        assert!(sala.esta_activa());
    }

    #[test]
    fn crear_sala_con_nombre_vacio() {
        let result = Sala::new("123".to_string(), "".to_string(), 10);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SalaError::NombreVacio);
    }

    #[test]
    fn crear_sala_con_nombre_solo_espacios() {
        let result = Sala::new("123".to_string(), "   ".to_string(), 10);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SalaError::NombreVacio);
    }

    #[test]
    fn crear_sala_con_nombre_demasiado_largo() {
        let nombre_largo = "a".repeat(101);
        let result = Sala::new("123".to_string(), nombre_largo, 10);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SalaError::NombreDemasiadoLargo);
    }

    #[test]
    fn crear_sala_con_capacidad_cero() {
        let result = Sala::new("123".to_string(), "Sala 1".to_string(), 0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SalaError::CapacidadInvalida);
    }

    #[test]
    fn crear_sala_con_capacidad_excesiva() {
        let result = Sala::new("123".to_string(), "Sala 1".to_string(), 1001);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SalaError::CapacidadInvalida);
    }

    #[test]
    fn activar_sala() {
        let mut sala = Sala::new("123".to_string(), "Sala 1".to_string(), 10).unwrap();
        sala.desactivar();
        assert!(!sala.esta_activa());

        sala.activar();
        assert!(sala.esta_activa());
    }

    #[test]
    fn desactivar_sala() {
        let mut sala = Sala::new("123".to_string(), "Sala 1".to_string(), 10).unwrap();
        assert!(sala.esta_activa());

        sala.desactivar();
        assert!(!sala.esta_activa());
    }

    #[test]
    fn nombre_trimea_espacios() {
        let sala = Sala::new("123".to_string(), "  Sala con espacios  ".to_string(), 10).unwrap();

        assert_eq!(sala.nombre(), "Sala con espacios");
    }
}
