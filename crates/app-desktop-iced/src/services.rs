use tonic::transport::Channel;
use tonic::Request;

use reservas_grpc::proto::reserva_service_client::ReservaServiceClient;
use reservas_grpc::proto::{
    CancelarReservaRequest, CrearReservaRequest, ListarReservasRequest, Reserva as ProtoReserva,
};
use salas_grpc::proto::sala_service_client::SalaServiceClient;
use salas_grpc::proto::{
    ActivarSalaRequest, CrearSalaRequest, DesactivarSalaRequest, ListarSalasRequest,
};
use usuarios_grpc::proto::usuario_service_client::UsuarioServiceClient;
use usuarios_grpc::proto::{LoginRequest, LoginResponse};

use crate::models::{SalaDto, GRPC_URL};

// ========== Cliente gRPC compartido ==========

async fn get_usuarios_client() -> Result<UsuarioServiceClient<Channel>, String> {
    UsuarioServiceClient::connect(GRPC_URL)
        .await
        .map_err(|e| format!("Error de conexión gRPC: {}", e))
}

async fn get_salas_client() -> Result<SalaServiceClient<Channel>, String> {
    SalaServiceClient::connect(GRPC_URL)
        .await
        .map_err(|e| format!("Error de conexión gRPC: {}", e))
}

async fn get_reservas_client() -> Result<ReservaServiceClient<Channel>, String> {
    ReservaServiceClient::connect(GRPC_URL)
        .await
        .map_err(|e| format!("Error de conexión gRPC: {}", e))
}

// ========== Servicios de Usuarios ==========

pub async fn login(email: String, password: String) -> Result<LoginResponse, String> {
    let mut client = get_usuarios_client().await?;

    let request = Request::new(LoginRequest { email, password });

    client
        .login(request)
        .await
        .map(|response| response.into_inner())
        .map_err(|e| format!("Error al hacer login: {}", e))
}

// ========== Servicios de Salas ==========

pub async fn listar_salas() -> Result<Vec<SalaDto>, String> {
    let mut client = get_salas_client().await?;

    let request = Request::new(ListarSalasRequest {});

    client
        .listar_salas(request)
        .await
        .map(|response| response.into_inner().salas)
        .map_err(|e| format!("Error al listar salas: {}", e))
}

pub async fn crear_sala(nombre: String, capacidad: u32) -> Result<SalaDto, String> {
    let mut client = get_salas_client().await?;

    let request = Request::new(CrearSalaRequest { nombre, capacidad });

    client
        .crear_sala(request)
        .await
        .map(|response| response.into_inner())
        .map_err(|e| format!("Error al crear sala: {}", e))
}

pub async fn activar_sala(id: String) -> Result<SalaDto, String> {
    let mut client = get_salas_client().await?;

    let request = Request::new(ActivarSalaRequest { id });

    client
        .activar_sala(request)
        .await
        .map(|response| response.into_inner())
        .map_err(|e| format!("Error al activar sala: {}", e))
}

pub async fn desactivar_sala(id: String) -> Result<SalaDto, String> {
    let mut client = get_salas_client().await?;

    let request = Request::new(DesactivarSalaRequest { id });

    client
        .desactivar_sala(request)
        .await
        .map(|response| response.into_inner())
        .map_err(|e| format!("Error al desactivar sala: {}", e))
}

// ========== Servicios de Reservas ==========

pub async fn listar_reservas() -> Result<Vec<ProtoReserva>, String> {
    let mut client = get_reservas_client().await?;

    let request = Request::new(ListarReservasRequest {});

    client
        .listar_reservas(request)
        .await
        .map(|response| response.into_inner().reservas)
        .map_err(|e| format!("Error al listar reservas: {}", e))
}

pub async fn crear_reserva(
    sala_id: String,
    usuario_id: String,
    fecha_inicio: String,
    fecha_fin: String,
) -> Result<ProtoReserva, String> {
    let mut client = get_reservas_client().await?;

    let request = Request::new(CrearReservaRequest {
        sala_id,
        usuario_id,
        fecha_inicio,
        fecha_fin,
    });

    client
        .crear_reserva(request)
        .await
        .map_err(|e| format!("Error al crear reserva: {}", e))?
        .into_inner()
        .reserva
        .ok_or_else(|| "Respuesta sin reserva".to_string())
}

pub async fn cancelar_reserva(id: String) -> Result<ProtoReserva, String> {
    let mut client = get_reservas_client().await?;

    let request = Request::new(CancelarReservaRequest { id });

    client
        .cancelar_reserva(request)
        .await
        .map_err(|e| format!("Error al cancelar reserva: {}", e))?
        .into_inner()
        .reserva
        .ok_or_else(|| "Respuesta sin reserva".to_string())
}

pub fn grpc_url() -> &'static str {
    GRPC_URL
}
