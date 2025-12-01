use crate::auth::{admin_middleware, auth_middleware};
use crate::handlers::{
    activar_sala, crear_sala, desactivar_sala, listar_salas, obtener_sala, SharedSalaService,
};
use crate::openapi::ApiDoc;
use axum::middleware;
use axum::routing::{post, put};
use axum::{routing::get, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Rutas de salas SIN autenticación
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

/// Rutas de salas CON autenticación
/// - GET: Requiere autenticación
/// - POST/PUT: Requieren rol de administrador
pub fn salas_routes_with_auth(service: SharedSalaService) -> Router {
    let openapi = ApiDoc::openapi();

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api/api-docs/openapi.json", openapi.clone()))
        // Ruta para servir el OpenAPI JSON (sin auth)
        .route(
            "/api-docs/openapi.json",
            get(|| async move { axum::Json(openapi) }),
        )
        // Rutas de lectura: requieren autenticación
        .route(
            "/salas",
            get(listar_salas).layer(middleware::from_fn(auth_middleware)),
        )
        .route(
            "/salas/{id}",
            get(obtener_sala).layer(middleware::from_fn(auth_middleware)),
        )
        // Rutas de escritura: requieren rol de admin
        .route(
            "/salas",
            post(crear_sala).layer(middleware::from_fn(admin_middleware)),
        )
        .route(
            "/salas/{id}/activar",
            put(activar_sala).layer(middleware::from_fn(admin_middleware)),
        )
        .route(
            "/salas/{id}/desactivar",
            put(desactivar_sala).layer(middleware::from_fn(admin_middleware)),
        )
        .with_state(service)
}
