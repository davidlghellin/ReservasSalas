use crate::{CrearSalaRequest, SalaResponse};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use salas_application::SalaService;
use salas_domain::SalaError;
use std::sync::Arc;
use tracing::log::{debug, error, info};

pub type SharedSalaService = Arc<dyn SalaService + Send + Sync>;

pub async fn crear_sala(
    State(service): State<SharedSalaService>,
    Json(request): Json<CrearSalaRequest>,
) -> Result<(StatusCode, Json<SalaResponse>), AppError> {
    info!("Creando sala: nombre={}, capacidad={}", request.nombre, request.capacidad);

    let sala = service
        .crear_sala(request.nombre, request.capacidad)
        .await
        .map_err(|e| {
            error!("Error al crear sala: {}", e);
            AppError(e)
        })?;

    debug!("Sala creada exitosamente: id={}", sala.id);
    Ok((StatusCode::CREATED, Json(sala.into())))
}

pub async fn listar_salas(
    State(service): State<SharedSalaService>,
) -> Result<Json<Vec<SalaResponse>>, AppError> {
    let salas = service.listar_salas().await?;
    let response: Vec<SalaResponse> = salas.iter().map(Into::into).collect();
    Ok(Json(response))
}

pub async fn obtener_sala(
    State(service): State<SharedSalaService>,
    Path(id): Path<String>,
) -> Result<Json<SalaResponse>, AppError> {
    let sala = service
        .obtener_sala(&id)
        .await?
        .ok_or(SalaError::NoEncontrada)?;
    let response: SalaResponse = sala.into();
    Ok(Json(response))
}

pub async fn activar_sala(
    State(service): State<SharedSalaService>,
    Path(id): Path<String>,
) -> Result<Json<SalaResponse>, AppError> {
    let sala = service.activar_sala(&id).await?;
    let response: SalaResponse = sala.into();
    Ok(Json(response))
}

pub async fn desactivar_sala(
    State(service): State<SharedSalaService>,
    Path(id): Path<String>,
) -> Result<Json<SalaResponse>, AppError> {
    let sala = service.desactivar_sala(&id).await?;
    let response: SalaResponse = sala.into();
    Ok(Json(response))
}
pub struct AppError(pub SalaError);

impl From<SalaError> for AppError {
    fn from(err: SalaError) -> Self {
        AppError(err)
    }
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self.0 {
            SalaError::NombreVacio
            | SalaError::NombreDemasiadoLargo
            | SalaError::CapacidadInvalida => (StatusCode::BAD_REQUEST, self.0.to_string()),
            SalaError::NoEncontrada => (StatusCode::NOT_FOUND, self.0.to_string()),
            SalaError::ErrorRepositorio(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string())
            }
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
