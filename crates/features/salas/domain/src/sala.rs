use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sala {
    pub id: String,
    pub nombre: String,
    pub capacidad: u32,
    pub activa: bool,
}

impl Sala {
    pub fn new(id: String, nombre: String, capacidad: u32) -> Result<Self, String> {
        if nombre.trim().is_empty() {
            return Err("El nombre no puede estar vacío".to_string());
        }
        if capacidad == 0 {
            return Err("La capacidad debe ser mayor a 0".to_string());
        }

        Ok(Self {
            id,
            nombre,
            capacidad,
            activa: true,
        })
    }

    pub fn desactivar(&mut self) {
        self.activa = false;
    }

    pub fn activar(&mut self) {
        self.activa = true;
    }
}