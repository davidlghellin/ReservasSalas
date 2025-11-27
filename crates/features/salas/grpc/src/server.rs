use std::sync::Arc;
use tonic::{Request, Response, Status};

use salas_application::SalaService;
use salas_domain::SalaError;

use crate::proto::{
    sala_service_server::{SalaService as SalaServiceTrait, SalaServiceServer},
    ActivarSalaRequest, CrearSalaRequest, DesactivarSalaRequest, ListarSalasRequest,
    ListarSalasResponse, ObtenerSalaRequest, SalaResponse,
};

pub struct SalaGrpcServer {
    service: Arc<dyn SalaService + Send + Sync>,
}

impl SalaGrpcServer {
    pub fn new(service: Arc<dyn SalaService + Send + Sync>) -> Self {
        Self { service }
    }

    pub fn into_service(self) -> SalaServiceServer<Self> {
        SalaServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl SalaServiceTrait for SalaGrpcServer {
    async fn crear_sala(
        &self,
        request: Request<CrearSalaRequest>,
    ) -> Result<Response<SalaResponse>, Status> {
        let req = request.into_inner();

        let sala = self
            .service
            .crear_sala(req.nombre, req.capacidad)
            .await
            .map_err(sala_error_to_status)?;

        Ok(Response::new(SalaResponse {
            id: sala.id().to_string(),
            nombre: sala.nombre().to_string(),
            capacidad: sala.capacidad(),
            activa: sala.activa,
        }))
    }

    async fn obtener_sala(
        &self,
        request: Request<ObtenerSalaRequest>,
    ) -> Result<Response<SalaResponse>, Status> {
        let req = request.into_inner();

        let sala = self
            .service
            .obtener_sala(&req.id)
            .await
            .map_err(sala_error_to_status)?
            .ok_or_else(|| Status::not_found("Sala no encontrada"))?;

        Ok(Response::new(SalaResponse {
            id: sala.id().to_string(),
            nombre: sala.nombre().to_string(),
            capacidad: sala.capacidad(),
            activa: sala.activa,
        }))
    }

    async fn listar_salas(
        &self,
        _request: Request<ListarSalasRequest>,
    ) -> Result<Response<ListarSalasResponse>, Status> {
        let salas = self
            .service
            .listar_salas()
            .await
            .map_err(sala_error_to_status)?;

        let salas_response: Vec<SalaResponse> = salas
            .into_iter()
            .map(|s| SalaResponse {
                id: s.id().to_string(),
                nombre: s.nombre().to_string(),
                capacidad: s.capacidad(),
                activa: s.activa,
            })
            .collect();

        Ok(Response::new(ListarSalasResponse {
            salas: salas_response,
        }))
    }

    async fn activar_sala(
        &self,
        request: Request<ActivarSalaRequest>,
    ) -> Result<Response<SalaResponse>, Status> {
        let req = request.into_inner();

        let sala = self
            .service
            .activar_sala(&req.id)
            .await
            .map_err(sala_error_to_status)?;

        Ok(Response::new(SalaResponse {
            id: sala.id().to_string(),
            nombre: sala.nombre().to_string(),
            capacidad: sala.capacidad(),
            activa: sala.activa,
        }))
    }

    async fn desactivar_sala(
        &self,
        request: Request<DesactivarSalaRequest>,
    ) -> Result<Response<SalaResponse>, Status> {
        let req = request.into_inner();

        let sala = self
            .service
            .desactivar_sala(&req.id)
            .await
            .map_err(sala_error_to_status)?;

        Ok(Response::new(SalaResponse {
            id: sala.id().to_string(),
            nombre: sala.nombre().to_string(),
            capacidad: sala.capacidad(),
            activa: sala.activa,
        }))
    }
}

// Helper para convertir SalaError a Status de gRPC
fn sala_error_to_status(error: SalaError) -> Status {
    match error {
        SalaError::NombreVacio | SalaError::NombreDemasiadoLargo | SalaError::CapacidadInvalida => {
            Status::invalid_argument(error.to_string())
        }
        SalaError::NoEncontrada => Status::not_found(error.to_string()),
        SalaError::ErrorRepositorio(_) => Status::internal(error.to_string()),
        SalaError::Validacion(msgs) => Status::invalid_argument(msgs.join("; ")),
    }
}
