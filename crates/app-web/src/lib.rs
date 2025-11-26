// Web UI - Adaptador de Entrada (interfaz web con HTML)
// Este adaptador sirve páginas HTML usando templates Askama

pub mod handlers;
pub mod routes;
pub mod templates;

pub use routes::crear_router_web;
