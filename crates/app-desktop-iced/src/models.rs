use salas_grpc::proto::SalaResponse;

pub const GRPC_URL: &str = "http://localhost:50051";

// Alias para simplificar el código de la UI
pub type SalaDto = SalaResponse;

#[derive(Debug, Clone)]
pub struct UsuarioInfo {
    pub id: String,
    pub nombre: String,
    pub email: String,
    pub rol: String,
}
