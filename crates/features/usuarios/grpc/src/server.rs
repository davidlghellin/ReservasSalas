use std::sync::Arc;
use tonic::{Request, Response, Status};

use usuarios_application::{AuthService, UsuarioService};
use usuarios_domain::{Rol, UsuarioError};

use crate::auth::{extract_admin_user, extract_auth_user};

use crate::proto::{
    usuario_service_server::{UsuarioService as UsuarioServiceTrait, UsuarioServiceServer},
    ActivarUsuarioRequest, ActivarUsuarioResponse, ActualizarNombreRequest, ActualizarRolRequest,
    ChangePasswordRequest, ChangePasswordResponse, DesactivarUsuarioRequest,
    DesactivarUsuarioResponse, ListarUsuariosRequest, ListarUsuariosResponse, LoginRequest,
    LoginResponse, ObtenerUsuarioRequest, RegisterRequest, RegisterResponse, UsuarioPublico,
    UsuarioPublicoResponse, ValidateTokenRequest, ValidateTokenResponse,
};

pub struct UsuarioGrpcServer {
    auth_service: Arc<dyn AuthService + Send + Sync>,
    usuario_service: Arc<dyn UsuarioService + Send + Sync>,
}

impl UsuarioGrpcServer {
    pub fn new(
        auth_service: Arc<dyn AuthService + Send + Sync>,
        usuario_service: Arc<dyn UsuarioService + Send + Sync>,
    ) -> Self {
        Self {
            auth_service,
            usuario_service,
        }
    }

    pub fn into_service(self) -> UsuarioServiceServer<Self> {
        UsuarioServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl UsuarioServiceTrait for UsuarioGrpcServer {
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();

        let login_response = self
            .auth_service
            .login(req.email, req.password)
            .await
            .map_err(usuario_error_to_status)?;

        Ok(Response::new(LoginResponse {
            token: login_response.token,
            usuario: Some(UsuarioPublico {
                id: login_response.usuario.id,
                nombre: login_response.usuario.nombre,
                email: login_response.usuario.email,
                rol: login_response.usuario.rol.as_str().to_string(),
                created_at: login_response.usuario.created_at.to_rfc3339(),
                activo: login_response.usuario.activo,
            }),
        }))
    }

    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();

        // Parsear rol si viene
        let rol = req.rol.as_ref().and_then(|r| Rol::from_str_opt(r));

        let register_response = self
            .auth_service
            .register(req.nombre, req.email, req.password, rol)
            .await
            .map_err(usuario_error_to_status)?;

        Ok(Response::new(RegisterResponse {
            token: register_response.token,
            usuario: Some(UsuarioPublico {
                id: register_response.usuario.id,
                nombre: register_response.usuario.nombre,
                email: register_response.usuario.email,
                rol: register_response.usuario.rol.as_str().to_string(),
                created_at: register_response.usuario.created_at.to_rfc3339(),
                activo: register_response.usuario.activo,
            }),
        }))
    }

    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let req = request.into_inner();

        let usuario = self
            .auth_service
            .validate_token(req.token)
            .await
            .map_err(usuario_error_to_status)?;

        Ok(Response::new(ValidateTokenResponse {
            usuario: Some(UsuarioPublico {
                id: usuario.id,
                nombre: usuario.nombre,
                email: usuario.email,
                rol: usuario.rol.as_str().to_string(),
                created_at: usuario.created_at.to_rfc3339(),
                activo: usuario.activo,
            }),
        }))
    }

    async fn change_password(
        &self,
        request: Request<ChangePasswordRequest>,
    ) -> Result<Response<ChangePasswordResponse>, Status> {
        // Validar autenticación
        let auth_user = extract_auth_user(&request)?;
        let req = request.into_inner();

        // Solo se puede cambiar la propia contraseña
        if auth_user.user_id != req.user_id {
            return Err(Status::permission_denied(
                "Solo puedes cambiar tu propia contraseña",
            ));
        }

        self.auth_service
            .change_password(req.user_id, req.old_password, req.new_password)
            .await
            .map_err(usuario_error_to_status)?;

        Ok(Response::new(ChangePasswordResponse {
            success: true,
            message: "Contraseña cambiada exitosamente".to_string(),
        }))
    }

    async fn listar_usuarios(
        &self,
        request: Request<ListarUsuariosRequest>,
    ) -> Result<Response<ListarUsuariosResponse>, Status> {
        // Requiere rol de administrador
        extract_admin_user(&request)?;

        let usuarios = self
            .usuario_service
            .listar_usuarios()
            .await
            .map_err(usuario_error_to_status)?;

        let usuarios_response: Vec<UsuarioPublico> = usuarios
            .into_iter()
            .map(|u| UsuarioPublico {
                id: u.id,
                nombre: u.nombre,
                email: u.email,
                rol: u.rol.as_str().to_string(),
                created_at: u.created_at.to_rfc3339(),
                activo: u.activo,
            })
            .collect();

        Ok(Response::new(ListarUsuariosResponse {
            usuarios: usuarios_response,
        }))
    }

    async fn obtener_usuario(
        &self,
        request: Request<ObtenerUsuarioRequest>,
    ) -> Result<Response<UsuarioPublicoResponse>, Status> {
        // Requiere autenticación
        extract_auth_user(&request)?;

        let req = request.into_inner();

        let usuario = self
            .usuario_service
            .obtener_usuario(req.id)
            .await
            .map_err(usuario_error_to_status)?;

        Ok(Response::new(UsuarioPublicoResponse {
            usuario: Some(UsuarioPublico {
                id: usuario.id,
                nombre: usuario.nombre,
                email: usuario.email,
                rol: usuario.rol.as_str().to_string(),
                created_at: usuario.created_at.to_rfc3339(),
                activo: usuario.activo,
            }),
        }))
    }

    async fn actualizar_nombre(
        &self,
        request: Request<ActualizarNombreRequest>,
    ) -> Result<Response<UsuarioPublicoResponse>, Status> {
        // Validar autenticación
        let auth_user = extract_auth_user(&request)?;
        let req = request.into_inner();

        // Solo se puede actualizar el propio nombre
        if auth_user.user_id != req.user_id {
            return Err(Status::permission_denied(
                "Solo puedes actualizar tu propio nombre",
            ));
        }

        let usuario = self
            .usuario_service
            .actualizar_nombre(req.user_id, req.nuevo_nombre)
            .await
            .map_err(usuario_error_to_status)?;

        Ok(Response::new(UsuarioPublicoResponse {
            usuario: Some(UsuarioPublico {
                id: usuario.id,
                nombre: usuario.nombre,
                email: usuario.email,
                rol: usuario.rol.as_str().to_string(),
                created_at: usuario.created_at.to_rfc3339(),
                activo: usuario.activo,
            }),
        }))
    }

    async fn actualizar_rol(
        &self,
        request: Request<ActualizarRolRequest>,
    ) -> Result<Response<UsuarioPublicoResponse>, Status> {
        // Requiere rol de administrador
        let auth_user = extract_admin_user(&request)?;
        let req = request.into_inner();

        let nuevo_rol = Rol::from_str_opt(&req.nuevo_rol)
            .ok_or_else(|| Status::invalid_argument("Rol inválido"))?;

        let usuario = self
            .usuario_service
            .actualizar_rol(auth_user.user_id, req.user_id, nuevo_rol)
            .await
            .map_err(usuario_error_to_status)?;

        Ok(Response::new(UsuarioPublicoResponse {
            usuario: Some(UsuarioPublico {
                id: usuario.id,
                nombre: usuario.nombre,
                email: usuario.email,
                rol: usuario.rol.as_str().to_string(),
                created_at: usuario.created_at.to_rfc3339(),
                activo: usuario.activo,
            }),
        }))
    }

    async fn desactivar_usuario(
        &self,
        request: Request<DesactivarUsuarioRequest>,
    ) -> Result<Response<DesactivarUsuarioResponse>, Status> {
        // Requiere rol de administrador
        let auth_user = extract_admin_user(&request)?;
        let req = request.into_inner();

        self.usuario_service
            .desactivar_usuario(auth_user.user_id, req.user_id)
            .await
            .map_err(usuario_error_to_status)?;

        Ok(Response::new(DesactivarUsuarioResponse {
            success: true,
            message: "Usuario desactivado exitosamente".to_string(),
        }))
    }

    async fn activar_usuario(
        &self,
        request: Request<ActivarUsuarioRequest>,
    ) -> Result<Response<ActivarUsuarioResponse>, Status> {
        // Requiere rol de administrador
        let auth_user = extract_admin_user(&request)?;
        let req = request.into_inner();

        self.usuario_service
            .activar_usuario(auth_user.user_id, req.user_id)
            .await
            .map_err(usuario_error_to_status)?;

        Ok(Response::new(ActivarUsuarioResponse {
            success: true,
            message: "Usuario activado exitosamente".to_string(),
        }))
    }
}

// Convertir errores de dominio a errores de gRPC
fn usuario_error_to_status(error: UsuarioError) -> Status {
    match error {
        UsuarioError::EmailInvalido(_) => Status::invalid_argument(error.mensaje_usuario()),
        UsuarioError::EmailDuplicado(_) => Status::already_exists(error.mensaje_usuario()),
        UsuarioError::NombreVacio => Status::invalid_argument(error.mensaje_usuario()),
        UsuarioError::NombreLongitudInvalida { .. } => {
            Status::invalid_argument(error.mensaje_usuario())
        }
        UsuarioError::ContrasenaDemasiadoCorta { .. } => {
            Status::invalid_argument(error.mensaje_usuario())
        }
        UsuarioError::UsuarioNoEncontrado(_) => Status::not_found(error.mensaje_usuario()),
        UsuarioError::CredencialesInvalidas => Status::unauthenticated(error.mensaje_usuario()),
        UsuarioError::PermisosDenegados => Status::permission_denied(error.mensaje_usuario()),
        UsuarioError::ErrorRepositorio(msg) => Status::internal(msg),
        UsuarioError::ValidacionError(msg) => Status::invalid_argument(msg),
    }
}
