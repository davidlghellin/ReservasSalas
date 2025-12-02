use reservas_grpc::proto::{
    reserva_service_client::ReservaServiceClient, CancelarReservaRequest, CrearReservaRequest,
    ListarReservasRequest, Reserva as ProtoReserva,
};
use salas_grpc::proto::{
    sala_service_client::SalaServiceClient, ActivarSalaRequest, CrearSalaRequest,
    DesactivarSalaRequest, ListarSalasRequest,
};
use tonic::{metadata::MetadataValue, Request};
use usuarios_grpc::proto::{usuario_service_client::UsuarioServiceClient, LoginRequest};

use crate::models::{SalaDto, UsuarioInfo, GRPC_URL};

// ========== Servicios de Usuarios ==========

pub async fn login_usuario(
    email: &str,
    password: &str,
) -> Result<(UsuarioInfo, String), String> {
    let mut client = UsuarioServiceClient::connect(GRPC_URL)
        .await
        .map_err(|e| format!("Error de conexión gRPC: {}", e))?;

    let request = Request::new(LoginRequest {
        email: email.to_string(),
        password: password.to_string(),
    });

    let response = client
        .login(request)
        .await
        .map_err(|e| format!("Error al hacer login: {}", e))?;

    let login_response = response.into_inner();
    let usuario_proto = login_response
        .usuario
        .ok_or_else(|| "Respuesta sin usuario".to_string())?;

    let usuario = UsuarioInfo {
        id: usuario_proto.id,
        nombre: usuario_proto.nombre,
        email: usuario_proto.email,
        rol: usuario_proto.rol,
    };

    Ok((usuario, login_response.token))
}

// ========== Servicios de Salas ==========

pub async fn listar_salas(token: &str) -> Result<Vec<SalaDto>, String> {
    let mut client = SalaServiceClient::connect(GRPC_URL)
        .await
        .map_err(|e| format!("Error de conexión gRPC: {}", e))?;

    let mut request = Request::new(ListarSalasRequest {});

    let auth_value = MetadataValue::try_from(format!("Bearer {}", token))
        .map_err(|e| format!("Error al crear header: {}", e))?;
    request.metadata_mut().insert("authorization", auth_value);

    let response = client
        .listar_salas(request)
        .await
        .map_err(|e| format!("Error gRPC: {}", e))?;

    let salas = response
        .into_inner()
        .salas
        .into_iter()
        .map(|s| SalaDto {
            id: s.id,
            nombre: s.nombre,
            capacidad: s.capacidad,
            activa: s.activa,
        })
        .collect();

    Ok(salas)
}

pub async fn crear_sala(nombre: &str, capacidad: u32, token: &str) -> Result<SalaDto, String> {
    let mut client = SalaServiceClient::connect(GRPC_URL)
        .await
        .map_err(|e| format!("Error de conexión gRPC: {}", e))?;

    let mut request = Request::new(CrearSalaRequest {
        nombre: nombre.to_string(),
        capacidad,
    });

    let auth_value = MetadataValue::try_from(format!("Bearer {}", token))
        .map_err(|e| format!("Error al crear header: {}", e))?;
    request.metadata_mut().insert("authorization", auth_value);

    let response = client
        .crear_sala(request)
        .await
        .map_err(|e| format!("Error gRPC: {}", e))?;

    let sala = response.into_inner();

    Ok(SalaDto {
        id: sala.id,
        nombre: sala.nombre,
        capacidad: sala.capacidad,
        activa: sala.activa,
    })
}

pub async fn activar_sala(id: &str, token: &str) -> Result<SalaDto, String> {
    let mut client = SalaServiceClient::connect(GRPC_URL)
        .await
        .map_err(|e| format!("Error de conexión gRPC: {}", e))?;

    let mut request = Request::new(ActivarSalaRequest { id: id.to_string() });

    let auth_value = MetadataValue::try_from(format!("Bearer {}", token))
        .map_err(|e| format!("Error al crear header: {}", e))?;
    request.metadata_mut().insert("authorization", auth_value);

    let response = client
        .activar_sala(request)
        .await
        .map_err(|e| format!("Error gRPC: {}", e))?;

    let sala = response.into_inner();

    Ok(SalaDto {
        id: sala.id,
        nombre: sala.nombre,
        capacidad: sala.capacidad,
        activa: sala.activa,
    })
}

pub async fn desactivar_sala(id: &str, token: &str) -> Result<SalaDto, String> {
    let mut client = SalaServiceClient::connect(GRPC_URL)
        .await
        .map_err(|e| format!("Error de conexión gRPC: {}", e))?;

    let mut request = Request::new(DesactivarSalaRequest { id: id.to_string() });

    let auth_value = MetadataValue::try_from(format!("Bearer {}", token))
        .map_err(|e| format!("Error al crear header: {}", e))?;
    request.metadata_mut().insert("authorization", auth_value);

    let response = client
        .desactivar_sala(request)
        .await
        .map_err(|e| format!("Error gRPC: {}", e))?;

    let sala = response.into_inner();

    Ok(SalaDto {
        id: sala.id,
        nombre: sala.nombre,
        capacidad: sala.capacidad,
        activa: sala.activa,
    })
}

// ========== Servicios de Reservas ==========

pub async fn listar_reservas(token: &str) -> Result<Vec<ProtoReserva>, String> {
    let mut client = ReservaServiceClient::connect(GRPC_URL)
        .await
        .map_err(|e| format!("Error de conexión gRPC: {}", e))?;

    let mut request = Request::new(ListarReservasRequest {});

    let auth_value = MetadataValue::try_from(format!("Bearer {}", token))
        .map_err(|e| format!("Error al crear header: {}", e))?;
    request.metadata_mut().insert("authorization", auth_value);

    let response = client
        .listar_reservas(request)
        .await
        .map_err(|e| format!("Error gRPC: {}", e))?;

    Ok(response.into_inner().reservas)
}

pub async fn crear_reserva(
    sala_id: &str,
    usuario_id: &str,
    fecha_inicio: &str,
    fecha_fin: &str,
    token: &str,
) -> Result<ProtoReserva, String> {
    let mut client = ReservaServiceClient::connect(GRPC_URL)
        .await
        .map_err(|e| format!("Error de conexión gRPC: {}", e))?;

    let mut request = Request::new(CrearReservaRequest {
        sala_id: sala_id.to_string(),
        usuario_id: usuario_id.to_string(),
        fecha_inicio: fecha_inicio.to_string(),
        fecha_fin: fecha_fin.to_string(),
    });

    let auth_value = MetadataValue::try_from(format!("Bearer {}", token))
        .map_err(|e| format!("Error al crear header: {}", e))?;
    request.metadata_mut().insert("authorization", auth_value);

    let response = client
        .crear_reserva(request)
        .await
        .map_err(|e| format!("Error gRPC: {}", e))?;

    response
        .into_inner()
        .reserva
        .ok_or_else(|| "Respuesta sin reserva".to_string())
}

pub async fn cancelar_reserva(id: &str, token: &str) -> Result<ProtoReserva, String> {
    let mut client = ReservaServiceClient::connect(GRPC_URL)
        .await
        .map_err(|e| format!("Error de conexión gRPC: {}", e))?;

    let mut request = Request::new(CancelarReservaRequest { id: id.to_string() });

    let auth_value = MetadataValue::try_from(format!("Bearer {}", token))
        .map_err(|e| format!("Error al crear header: {}", e))?;
    request.metadata_mut().insert("authorization", auth_value);

    let response = client
        .cancelar_reserva(request)
        .await
        .map_err(|e| format!("Error gRPC: {}", e))?;

    response
        .into_inner()
        .reserva
        .ok_or_else(|| "Respuesta sin reserva".to_string())
}
