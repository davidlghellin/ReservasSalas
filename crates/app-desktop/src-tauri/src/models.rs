use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SalaDto {
    pub id: String,
    pub nombre: String,
    pub capacidad: u32,
    pub activa: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CrearSalaRequest {
    pub nombre: String,
    pub capacidad: u32,
}

