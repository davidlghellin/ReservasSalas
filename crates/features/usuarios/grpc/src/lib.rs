pub mod auth;
pub mod server;

pub use auth::{extract_admin_user, extract_auth_user, AuthUser, RequestAuthExt};
pub use server::UsuarioGrpcServer;

// Re-exportar los tipos generados por tonic
pub mod proto {
    tonic::include_proto!("usuario");

    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("usuario_descriptor");
}
