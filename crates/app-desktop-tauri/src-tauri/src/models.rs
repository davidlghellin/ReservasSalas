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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsuarioInfo {
    pub id: String,
    pub nombre: String,
    pub email: String,
    pub rol: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginResponse {
    pub token: String,
    pub usuario: UsuarioInfo,
}

