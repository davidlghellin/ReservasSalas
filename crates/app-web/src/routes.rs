// Configuración de rutas para la interfaz web

use crate::handlers;
use axum::{
    routing::{get, post},
    Router,
};
use salas_application::SalaService;
use std::sync::Arc;
use tower_http::services::ServeDir;

/// Crea el router con todas las rutas de la interfaz web
pub fn crear_router_web(sala_service: Arc<dyn SalaService + Send + Sync>) -> Router {
    let static_files = ServeDir::new("crates/app-web/static");

    Router::new()
        .route("/", get(handlers::index::index))
        .route("/salas", get(handlers::sala::listar_salas_page))
        .route("/salas/nuevo", get(handlers::sala::nuevo_sala_form))
        .route("/salas/crear", post(handlers::sala::crear_sala_submit))
        .route("/salas/{id}/activar", post(handlers::sala::activar_sala))
        .route(
            "/salas/{id}/desactivar",
            post(handlers::sala::desactivar_sala),
        )
        .nest_service("/static", static_files)
        .with_state(sala_service)
}
