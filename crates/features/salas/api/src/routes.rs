use crate::handlers::{
    activar_sala, crear_sala, desactivar_sala, listar_salas, obtener_sala, SharedSalaService,
};
use axum::routing::{post, put};
use axum::{routing::get, Router};

pub fn salas_routes(service: SharedSalaService) -> Router {
    Router::new()
        .route("/", post(crear_sala).get(listar_salas))
        .route("/{id}", get(obtener_sala))
        .route("/{id}/activar", put(activar_sala))
        .route("/{id}/desactivar", put(desactivar_sala))
        .with_state(service)
}
