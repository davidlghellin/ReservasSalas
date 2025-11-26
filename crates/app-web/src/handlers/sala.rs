use crate::templates::{DisponibilidadTemplate, SalaFormTemplate, SalaView, SalasTemplate};
use askama::Template;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect};
use axum::Form;
use salas_application::SalaService;
use serde::Deserialize;
use std::sync::Arc;

pub async fn nuevo_sala_form() -> impl IntoResponse {
    Html(SalaFormTemplate.render().unwrap())
}
pub async fn disponibilidad_page() -> impl IntoResponse {
    Html(DisponibilidadTemplate.render().unwrap())
}

#[derive(Deserialize)]
pub struct CrearSalaForm {
    pub(crate) nombre: String,
    pub(crate) capacidad: u32,
}

pub async fn crear_sala_submit(
    State(service): State<Arc<dyn SalaService + Send + Sync>>,
    Form(form): Form<CrearSalaForm>,
) -> Result<Redirect, StatusCode> {
    service
        .crear_sala(form.nombre, form.capacidad)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(Redirect::to("/salas"))
}

pub async fn listar_salas_page(
    State(service): State<Arc<dyn SalaService + Send + Sync>>,
) -> Result<impl IntoResponse, StatusCode> {
    let salas = service
        .listar_salas()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let salas_view: Vec<SalaView> = salas
        .into_iter()
        .map(|s| SalaView {
            id: s.id,
            nombre: s.nombre,
            capacidad: s.capacidad,
            activa: s.activa,
        })
        .collect();

    Ok(Html(SalasTemplate { salas: salas_view }.render().unwrap()))
}

pub async fn activar_sala(
    State(service): State<Arc<dyn SalaService + Send + Sync>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    service
        .activar_sala(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Redirect::to("/salas"))
}

pub async fn desactivar_sala(
    State(service): State<Arc<dyn SalaService + Send + Sync>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    service
        .desactivar_sala(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Redirect::to("/salas"))
}
