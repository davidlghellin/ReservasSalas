use crate::backend::BackendApi;
use crate::logger::Logger;
use crate::models::{CrearSalaRequest, SalaDto};
use tauri::State;

#[tauri::command]
pub async fn crear_sala(
    request: CrearSalaRequest,
    backend: State<'_, BackendApi>,
    logger: State<'_, Logger>,
) -> Result<SalaDto, String> {
    logger.info(&format!("Creando sala: {}", request.nombre));
    match backend.crear_sala(request).await {
        Ok(sala) => {
            logger.info(&format!("Sala creada: {} (ID: {})", sala.nombre, sala.id));
            Ok(sala)
        }
        Err(e) => {
            logger.error(&format!("Error creando sala: {}", e));
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn listar_salas(
    backend: State<'_, BackendApi>,
    logger: State<'_, Logger>,
) -> Result<Vec<SalaDto>, String> {
    logger.debug("Listando salas");
    match backend.listar_salas().await {
        Ok(salas) => {
            logger.info(&format!("Listadas {} salas", salas.len()));
            Ok(salas)
        }
        Err(e) => {
            logger.error(&format!("Error listando salas: {}", e));
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn obtener_sala(
    id: String,
    backend: State<'_, BackendApi>,
    logger: State<'_, Logger>,
) -> Result<Option<SalaDto>, String> {
    logger.debug(&format!("Obteniendo sala: {}", id));
    match backend.obtener_sala(&id).await {
        Ok(sala) => {
            if sala.is_some() {
                logger.info(&format!("Sala encontrada: {}", id));
            } else {
                logger.info(&format!("Sala no encontrada: {}", id));
            }
            Ok(sala)
        }
        Err(e) => {
            logger.error(&format!("Error obteniendo sala {}: {}", id, e));
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn activar_sala(
    id: String,
    backend: State<'_, BackendApi>,
    logger: State<'_, Logger>,
) -> Result<SalaDto, String> {
    logger.info(&format!("Activando sala: {}", id));
    match backend.activar_sala(&id).await {
        Ok(sala) => {
            logger.info(&format!("Sala activada: {}", id));
            Ok(sala)
        }
        Err(e) => {
            logger.error(&format!("Error activando sala {}: {}", id, e));
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn desactivar_sala(
    id: String,
    backend: State<'_, BackendApi>,
    logger: State<'_, Logger>,
) -> Result<SalaDto, String> {
    logger.info(&format!("Desactivando sala: {}", id));
    match backend.desactivar_sala(&id).await {
        Ok(sala) => {
            logger.info(&format!("Sala desactivada: {}", id));
            Ok(sala)
        }
        Err(e) => {
            logger.error(&format!("Error desactivando sala {}: {}", id, e));
            Err(e)
        }
    }
}

/// Obtiene la ruta del archivo de logs
#[tauri::command]
pub fn get_log_path(logger: State<'_, Logger>) -> String {
    logger.log_path().display().to_string()
}

/// Login de usuario usando gRPC
#[tauri::command]
pub async fn login_usuario(
    request: crate::models::LoginRequest,
    backend: State<'_, BackendApi>,
    logger: State<'_, Logger>,
) -> Result<crate::models::LoginResponse, String> {
    use tonic::Request;
    use usuarios_grpc::proto::{
        usuario_service_client::UsuarioServiceClient, LoginRequest as GrpcLoginRequest,
    };

    logger.info(&format!("Iniciando login para: {}", request.email));

    let mut client = UsuarioServiceClient::connect("http://localhost:50051")
        .await
        .map_err(|e| {
            logger.error(&format!("Error de conexión gRPC: {}", e));
            format!("Error de conexión: {}", e)
        })?;

    let grpc_request = Request::new(GrpcLoginRequest {
        email: request.email.clone(),
        password: request.password,
    });

    let response = client.login(grpc_request).await.map_err(|e| {
        logger.error(&format!("Error en login gRPC: {}", e));
        format!("Error al hacer login: {}", e)
    })?;

    let login_response = response.into_inner();
    let usuario_proto = login_response.usuario.ok_or_else(|| {
        logger.error("Respuesta de login sin usuario");
        "Respuesta inválida del servidor".to_string()
    })?;

    let usuario = crate::models::UsuarioInfo {
        id: usuario_proto.id,
        nombre: usuario_proto.nombre,
        email: usuario_proto.email,
        rol: usuario_proto.rol,
    };

    logger.info(&format!("Login exitoso para: {}", usuario.email));

    // Guardar el token en el backend
    backend.set_token(Some(login_response.token.clone()));

    Ok(crate::models::LoginResponse {
        token: login_response.token,
        usuario,
    })
}

/// Limpia el token de autenticación
#[tauri::command]
pub fn logout_usuario(backend: State<'_, BackendApi>) {
    backend.set_token(None);
}
