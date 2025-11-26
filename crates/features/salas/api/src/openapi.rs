use crate::dtos::{CrearSalaRequest, ErrorResponse, SalaResponse, ValidationErrorResponse};
use crate::handlers;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::crear_sala,
        handlers::listar_salas,
        handlers::obtener_sala,
        handlers::activar_sala,
        handlers::desactivar_sala,
    ),
    components(
        schemas(CrearSalaRequest, SalaResponse, ErrorResponse, ValidationErrorResponse)
    ),
    tags(
        (name = "salas", description = "Gestión de salas de reuniones")
    ),
    info(
        title = "API de Salas",
        version = "1.0.0",
        description = "API REST para la gestión de salas de reuniones"
    ),
    servers(
        (url = "/api", description = "API base path")
    )
)]
pub struct ApiDoc;
