pub mod auth;
pub mod server;

pub use auth::{AuthUser, RequestAuthExt};
pub use server::ReservaGrpcServer;

// Re-exportar los tipos generados por tonic
pub mod proto {
    tonic::include_proto!("reserva");

    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("reserva_descriptor");
}
