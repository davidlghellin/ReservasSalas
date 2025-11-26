use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CrearSalaRequest {
    pub nombre: String,
    pub capacidad: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SalaResponse {
    pub id: String,
    pub nombre: String,
    pub capacidad: u32,
    pub activa: bool,
}
