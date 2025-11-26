use crate::handlers::{
    activar_sala, crear_sala, desactivar_sala, listar_salas, obtener_sala, SharedSalaService,
};
use crate::openapi::ApiDoc;
use axum::routing::{post, put};
use axum::{routing::get, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn salas_routes(service: SharedSalaService) -> Router {
    let openapi = ApiDoc::openapi();

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api/api-docs/openapi.json", openapi.clone()))
        // Ruta para servir el OpenAPI JSON
        .route(
            "/api-docs/openapi.json",
            get(|| async move { axum::Json(openapi) }),
        )
        .route("/salas", post(crear_sala).get(listar_salas))
        .route("/salas/{id}", get(obtener_sala))
        .route("/salas/{id}/activar", put(activar_sala))
        .route("/salas/{id}/desactivar", put(desactivar_sala))
        .with_state(service)
}
