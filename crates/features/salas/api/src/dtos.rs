use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CrearSalaRequest {
    #[schema(example = "Sala de Conferencias")]
    pub nombre: String,
    #[schema(example = 50, minimum = 1, maximum = 1000)]
    pub capacidad: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SalaResponse {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,
    #[schema(example = "Sala de Conferencias")]
    pub nombre: String,
    #[schema(example = 50)]
    pub capacidad: u32,
    #[schema(example = true)]
    pub activa: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    #[schema(example = "El nombre no puede estar vacío")]
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ValidationErrorResponse {
    pub errors: Vec<String>,
}
