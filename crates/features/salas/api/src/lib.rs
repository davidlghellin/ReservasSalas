pub mod dtos;
pub mod mapper;
pub use dtos::{CrearSalaRequest, SalaResponse};
pub use mapper::SalaMapper;

pub mod handlers;
pub mod openapi;
pub mod routes;

pub use openapi::ApiDoc;
