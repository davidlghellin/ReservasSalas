use serde::{Deserialize, Serialize};
use crate::error::SalaError;

const MAX_NOMBRE_LENGTH: usize = 100;
const MAX_CAPACIDAD: u32 = 1000;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sala {
    pub id: String,
    pub nombre: String,
    pub capacidad: u32,
    pub activa: bool,
}

impl Sala {
    pub fn new(id: String, nombre: String, capacidad: u32) -> Result<Self, SalaError> {
        Self::validar_nombre(&nombre)?;
        Self::validar_capacidad(capacidad)?;

        Ok(Self {
            id,
            nombre: nombre.trim().to_string(),
            capacidad,
            activa: true,
        })
    }

    fn validar_nombre(nombre: &str) -> Result<(), SalaError> {
        if nombre.trim().is_empty() {
            return Err(SalaError::NombreVacio);
        }
        if nombre.len() > MAX_NOMBRE_LENGTH {
            return Err(SalaError::NombreDemasiadoLargo);
        }
        Ok(())
    }

    fn validar_capacidad(capacidad: u32) -> Result<(), SalaError> {
        if capacidad == 0 || capacidad > MAX_CAPACIDAD {
            return Err(SalaError::CapacidadInvalida);
        }
        Ok(())
    }

    pub fn desactivar(&mut self) {
        self.activa = false;
    }

    pub fn activar(&mut self) {
        self.activa = true;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crear_sala_valida() {
        let sala = Sala::new(
            "123".to_string(),
            "Sala de Conferencias".to_string(),
            10,
        );

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
        let sala = Sala::new(
            "123".to_string(),
            "  Sala con espacios  ".to_string(),
            10,
        ).unwrap();

        assert_eq!(sala.nombre(), "Sala con espacios");
    }
}