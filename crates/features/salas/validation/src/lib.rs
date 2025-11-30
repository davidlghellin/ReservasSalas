//! Validaciones compartidas para las operaciones de salas
//!
//! Este crate proporciona validaciones reutilizables que pueden ser usadas
//! tanto en el frontend (Iced, Dioxus, etc.) como en el backend (servidor gRPC).
//!
//! # Ejemplo de uso
//!
//! ```rust
//! use salas_validation::{ValidarSala, SalaValidationError};
//! use salas_grpc::proto::CrearSalaRequest;
//!
//! let request = CrearSalaRequest {
//!     nombre: "Sala 101".to_string(),
//!     capacidad: 50,
//! };
//!
//! match request.validar() {
//!     Ok(()) => println!("Request válido"),
//!     Err(e) => eprintln!("Error de validación: {}", e.mensaje_usuario()),
//! }
//! ```

mod error;
mod sala;

// Exportar públicamente
pub use error::SalaValidationError;
pub use sala::{
    validar_capacidad, validar_id, validar_nombre, ValidarSala, CAPACIDAD_MAX, CAPACIDAD_MIN,
    NOMBRE_MAX_LENGTH, NOMBRE_MIN_LENGTH,
};
