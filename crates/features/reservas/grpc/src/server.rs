use crate::auth::RequestAuthExt;
use crate::proto::reserva_service_server::ReservaService as ReservaServiceTrait;
use crate::proto::{
    CancelarReservaRequest, CompletarReservaRequest, CrearReservaRequest,
    EstadoReserva as ProtoEstadoReserva, ListarReservasPorSalaRequest,
    ListarReservasPorUsuarioRequest, ListarReservasRequest, ListarReservasResponse,
    ObtenerReservaRequest, Reserva as ProtoReserva, ReservaResponse,
    VerificarDisponibilidadRequest, VerificarDisponibilidadResponse,
};
use chrono::{DateTime, Utc};
use reservas_application::ReservaService;
use reservas_domain::{EstadoReserva, Reserva};
use std::sync::Arc;
use tonic::{Request, Response, Status};

/// Servidor gRPC para el servicio de Reservas
pub struct ReservaGrpcServer<S: ReservaService> {
    service: Arc<S>,
}

impl<S: ReservaService> ReservaGrpcServer<S> {
    pub fn new(service: S) -> Self {
        Self {
            service: Arc::new(service),
        }
    }
}

// Funciones de conversión entre tipos de dominio y proto

fn reserva_to_proto(reserva: &Reserva) -> ProtoReserva {
    ProtoReserva {
        id: reserva.id().to_string(),
        sala_id: reserva.sala_id().to_string(),
        usuario_id: reserva.usuario_id().to_string(),
        fecha_inicio: reserva.fecha_inicio().to_rfc3339(),
        fecha_fin: reserva.fecha_fin().to_rfc3339(),
        estado: estado_to_proto(reserva.estado()),
        created_at: reserva.created_at().to_rfc3339(),
    }
}

fn estado_to_proto(estado: &EstadoReserva) -> i32 {
    match estado {
        EstadoReserva::Activa => ProtoEstadoReserva::Activa as i32,
        EstadoReserva::Cancelada => ProtoEstadoReserva::Cancelada as i32,
        EstadoReserva::Completada => ProtoEstadoReserva::Completada as i32,
    }
}

#[allow(clippy::result_large_err)]
fn parse_datetime(s: &str) -> Result<DateTime<Utc>, Status> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| Status::invalid_argument(format!("Fecha inválida: {}", e)))
}

#[tonic::async_trait]
impl<S: ReservaService + 'static> ReservaServiceTrait for ReservaGrpcServer<S> {
    async fn crear_reserva(
        &self,
        request: Request<CrearReservaRequest>,
    ) -> Result<Response<ReservaResponse>, Status> {
        // Validar autenticación
        request.require_auth_user()?;

        let req = request.into_inner();

        let fecha_inicio = parse_datetime(&req.fecha_inicio)?;
        let fecha_fin = parse_datetime(&req.fecha_fin)?;

        let reserva = self
            .service
            .crear_reserva(req.sala_id, req.usuario_id, fecha_inicio, fecha_fin)
            .await
            .map_err(|e| Status::internal(format!("Error al crear reserva: {}", e)))?;

        Ok(Response::new(ReservaResponse {
            reserva: Some(reserva_to_proto(&reserva)),
        }))
    }

    async fn obtener_reserva(
        &self,
        request: Request<ObtenerReservaRequest>,
    ) -> Result<Response<ReservaResponse>, Status> {
        // Validar autenticación
        request.require_auth_user()?;

        let req = request.into_inner();

        let reserva = self
            .service
            .obtener_reserva(&req.id)
            .await
            .map_err(|e| Status::internal(format!("Error al obtener reserva: {}", e)))?
            .ok_or_else(|| Status::not_found("Reserva no encontrada"))?;

        Ok(Response::new(ReservaResponse {
            reserva: Some(reserva_to_proto(&reserva)),
        }))
    }

    async fn listar_reservas(
        &self,
        request: Request<ListarReservasRequest>,
    ) -> Result<Response<ListarReservasResponse>, Status> {
        // Validar autenticación
        request.require_auth_user()?;

        let reservas = self
            .service
            .listar_reservas()
            .await
            .map_err(|e| Status::internal(format!("Error al listar reservas: {}", e)))?;

        let proto_reservas = reservas.iter().map(reserva_to_proto).collect();

        Ok(Response::new(ListarReservasResponse {
            reservas: proto_reservas,
        }))
    }

    async fn listar_reservas_por_sala(
        &self,
        request: Request<ListarReservasPorSalaRequest>,
    ) -> Result<Response<ListarReservasResponse>, Status> {
        // Validar autenticación
        request.require_auth_user()?;

        let req = request.into_inner();

        let reservas = self
            .service
            .listar_reservas_por_sala(&req.sala_id)
            .await
            .map_err(|e| Status::internal(format!("Error al listar reservas: {}", e)))?;

        let proto_reservas = reservas.iter().map(reserva_to_proto).collect();

        Ok(Response::new(ListarReservasResponse {
            reservas: proto_reservas,
        }))
    }

    async fn listar_reservas_por_usuario(
        &self,
        request: Request<ListarReservasPorUsuarioRequest>,
    ) -> Result<Response<ListarReservasResponse>, Status> {
        // Validar autenticación
        request.require_auth_user()?;

        let req = request.into_inner();

        let reservas = self
            .service
            .listar_reservas_por_usuario(&req.usuario_id)
            .await
            .map_err(|e| Status::internal(format!("Error al listar reservas: {}", e)))?;

        let proto_reservas = reservas.iter().map(reserva_to_proto).collect();

        Ok(Response::new(ListarReservasResponse {
            reservas: proto_reservas,
        }))
    }

    async fn cancelar_reserva(
        &self,
        request: Request<CancelarReservaRequest>,
    ) -> Result<Response<ReservaResponse>, Status> {
        // Validar autenticación
        request.require_auth_user()?;

        let req = request.into_inner();

        let reserva = self
            .service
            .cancelar_reserva(&req.id)
            .await
            .map_err(|e| Status::internal(format!("Error al cancelar reserva: {}", e)))?;

        Ok(Response::new(ReservaResponse {
            reserva: Some(reserva_to_proto(&reserva)),
        }))
    }

    async fn completar_reserva(
        &self,
        request: Request<CompletarReservaRequest>,
    ) -> Result<Response<ReservaResponse>, Status> {
        // Validar autenticación
        request.require_auth_user()?;

        let req = request.into_inner();

        let reserva = self
            .service
            .completar_reserva(&req.id)
            .await
            .map_err(|e| Status::internal(format!("Error al completar reserva: {}", e)))?;

        Ok(Response::new(ReservaResponse {
            reserva: Some(reserva_to_proto(&reserva)),
        }))
    }

    async fn verificar_disponibilidad(
        &self,
        request: Request<VerificarDisponibilidadRequest>,
    ) -> Result<Response<VerificarDisponibilidadResponse>, Status> {
        // Validar autenticación
        request.require_auth_user()?;

        let req = request.into_inner();

        let fecha_inicio = parse_datetime(&req.fecha_inicio)?;
        let fecha_fin = parse_datetime(&req.fecha_fin)?;

        let disponible = self
            .service
            .verificar_disponibilidad(&req.sala_id, fecha_inicio, fecha_fin)
            .await
            .map_err(|e| Status::internal(format!("Error al verificar disponibilidad: {}", e)))?;

        let mensaje = if disponible {
            "La sala está disponible en el horario solicitado".to_string()
        } else {
            "La sala no está disponible en el horario solicitado".to_string()
        };

        Ok(Response::new(VerificarDisponibilidadResponse {
            disponible,
            mensaje,
        }))
    }
}

#[cfg(test)]
mod grpc_unit_tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use reservas_domain::{EstadoReserva, Reserva as DomainReserva, ReservaError};

    /// Mock mínimo del `ReservaService` para tests unitarios del servidor gRPC.
    struct MockReservaService {
        reservas: Vec<DomainReserva>,
    }

    impl MockReservaService {
        fn new() -> Self {
            let ahora = Utc::now();
            let r = DomainReserva::from_existing(
                "r1".to_string(),
                "sala1".to_string(),
                "usuario1".to_string(),
                ahora,
                ahora + chrono::Duration::hours(1),
                EstadoReserva::Activa,
                ahora,
            );
            Self { reservas: vec![r] }
        }
    }

    #[async_trait]
    impl reservas_application::ReservaService for MockReservaService {
        async fn crear_reserva(
            &self,
            _sala_id: String,
            _usuario_id: String,
            _fecha_inicio: DateTime<Utc>,
            _fecha_fin: DateTime<Utc>,
        ) -> Result<DomainReserva, ReservaError> {
            // Devuelve la primera reserva como "creada" (mock)
            Ok(self.reservas[0].clone())
        }

        async fn obtener_reserva(&self, id: &str) -> Result<Option<DomainReserva>, ReservaError> {
            Ok(self.reservas.iter().find(|&r| r.id() == id).cloned())
        }

        async fn listar_reservas(&self) -> Result<Vec<DomainReserva>, ReservaError> {
            Ok(self.reservas.clone())
        }

        async fn listar_reservas_por_sala(
            &self,
            sala_id: &str,
        ) -> Result<Vec<DomainReserva>, ReservaError> {
            Ok(self
                .reservas
                .iter()
                .filter(|&r| r.sala_id() == sala_id)
                .cloned()
                .collect())
        }

        async fn listar_reservas_por_usuario(
            &self,
            usuario_id: &str,
        ) -> Result<Vec<DomainReserva>, ReservaError> {
            Ok(self
                .reservas
                .iter()
                .filter(|&r| r.usuario_id() == usuario_id)
                .cloned()
                .collect())
        }

        async fn cancelar_reserva(&self, id: &str) -> Result<DomainReserva, ReservaError> {
            self.obtener_reserva(id)
                .await?
                .ok_or(ReservaError::NoEncontrada)
        }

        async fn completar_reserva(&self, id: &str) -> Result<DomainReserva, ReservaError> {
            self.obtener_reserva(id)
                .await?
                .ok_or(ReservaError::NoEncontrada)
        }

        async fn verificar_disponibilidad(
            &self,
            _sala_id: &str,
            _fecha_inicio: DateTime<Utc>,
            _fecha_fin: DateTime<Utc>,
        ) -> Result<bool, ReservaError> {
            Ok(true)
        }
    }

    #[tokio::test]
    async fn servidor_listar_reservas_devuelve_lista() {
        let service = MockReservaService::new();
        let server = ReservaGrpcServer::new(service);

        // Generar token válido y añadirlo al metadata para pasar la autenticación
        let token = usuarios_auth::jwt::JwtService::generate_token(
            "test-user",
            "test@example.com",
            usuarios_domain::Rol::Usuario,
        )
        .expect("failed to generate token");
        let mut req = tonic::Request::new(ListarReservasRequest {});
        req.metadata_mut().insert(
            "authorization",
            tonic::metadata::MetadataValue::try_from(format!("Bearer {}", token)).unwrap(),
        );

        let resp = server
            .listar_reservas(req)
            .await
            .expect("listar_reservas falló");
        let inner = resp.into_inner();

        assert_eq!(inner.reservas.len(), 1);
        let proto = &inner.reservas[0];
        assert_eq!(proto.sala_id, "sala1");
        assert_eq!(proto.usuario_id, "usuario1");
        assert_eq!(proto.estado, ProtoEstadoReserva::Activa as i32);
    }

    #[tokio::test]
    async fn servidor_crear_reserva_devuelve_reserva() {
        let service = MockReservaService::new();
        let server = ReservaGrpcServer::new(service);

        let ahora = Utc::now();
        let mut req = tonic::Request::new(CrearReservaRequest {
            sala_id: "sala1".to_string(),
            usuario_id: "usuario1".to_string(),
            fecha_inicio: ahora.to_rfc3339(),
            fecha_fin: (ahora + chrono::Duration::hours(1)).to_rfc3339(),
        });

        let token = usuarios_auth::jwt::JwtService::generate_token(
            "test-user",
            "test@example.com",
            usuarios_domain::Rol::Usuario,
        )
        .expect("failed to generate token");
        req.metadata_mut().insert(
            "authorization",
            tonic::metadata::MetadataValue::try_from(format!("Bearer {}", token)).unwrap(),
        );

        let resp = server
            .crear_reserva(req)
            .await
            .expect("crear_reserva falló");
        let inner = resp.into_inner();
        let proto = inner.reserva.expect("no vino reserva");

        assert_eq!(proto.sala_id, "sala1");
        assert_eq!(proto.usuario_id, "usuario1");
        assert_eq!(proto.estado, ProtoEstadoReserva::Activa as i32);
    }
}
