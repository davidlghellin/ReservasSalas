pub mod server;

pub use server::SalaGrpcServer;

// Re-exportar los tipos generados por tonic
pub mod proto {
    tonic::include_proto!("sala");

    pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("sala_descriptor");
}
