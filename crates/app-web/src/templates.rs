// Definición de templates usando Askama

use askama::Template;

// ============= DTOs para templates =============

#[derive(Debug)]
pub struct SalaView {
    pub id: String,
    pub nombre: String,
    pub capacidad: u32,
    pub activa: bool,
}

// ============= Templates =============

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate;

#[derive(Template)]
#[template(path = "salas_form.html")]
pub struct SalaFormTemplate;

#[derive(Template)]
#[template(path = "disponibilidad.html")]
pub struct DisponibilidadTemplate;

#[derive(Template)]
#[template(path = "salas.html")]
pub struct SalasTemplate {
    pub salas: Vec<SalaView>,
}
