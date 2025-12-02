use serde::{Deserialize, Serialize};

pub const BACKEND_URL: &str = "http://localhost:3000/api";
pub const GRPC_URL: &str = "http://localhost:50051";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SalaDto {
    pub id: String,
    pub nombre: String,
    pub capacidad: u32,
    pub activa: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsuarioInfo {
    pub id: String,
    pub nombre: String,
    pub email: String,
    pub rol: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Login,
    Authenticated(UsuarioInfo),
}
