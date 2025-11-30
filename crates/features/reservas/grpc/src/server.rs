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
mod tests {
    use super::*;
    use chrono::Duration;
    use reservas_application::ReservaServiceImpl;
    use reservas_domain::Reserva;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Mock repository para testing
    struct MockReservaRepository {
        reservas: Arc<Mutex<HashMap<String, Reserva>>>,
    }

    impl MockReservaRepository {
        fn new() -> Self {
            Self {
                reservas: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait::async_trait]
    impl reservas_application::ReservaRepository for MockReservaRepository {
        async fn guardar(
            &self,
            reserva: &Reserva,
        ) -> Result<(), reservas_domain::ReservaError> {
            let mut reservas = self.reservas.lock().unwrap();
            reservas.insert(reserva.id().to_string(), reserva.clone());
            Ok(())
        }

        async fn obtener(
            &self,
            id: &str,
        ) -> Result<Option<Reserva>, reservas_domain::ReservaError> {
            let reservas = self.reservas.lock().unwrap();
            Ok(reservas.get(id).cloned())
        }

        async fn listar(&self) -> Result<Vec<Reserva>, reservas_domain::ReservaError> {
            let reservas = self.reservas.lock().unwrap();
            Ok(reservas.values().cloned().collect())
        }

        async fn listar_por_sala(
            &self,
            sala_id: &str,
        ) -> Result<Vec<Reserva>, reservas_domain::ReservaError> {
            let reservas = self.reservas.lock().unwrap();
            Ok(reservas
                .values()
                .filter(|r| r.sala_id() == sala_id)
                .cloned()
                .collect())
        }

        async fn listar_por_usuario(
            &self,
            usuario_id: &str,
        ) -> Result<Vec<Reserva>, reservas_domain::ReservaError> {
            let reservas = self.reservas.lock().unwrap();
            Ok(reservas
                .values()
                .filter(|r| r.usuario_id() == usuario_id)
                .cloned()
                .collect())
        }

        async fn listar_por_sala_y_rango(
            &self,
            sala_id: &str,
            inicio: DateTime<Utc>,
            fin: DateTime<Utc>,
        ) -> Result<Vec<Reserva>, reservas_domain::ReservaError> {
            let reservas = self.reservas.lock().unwrap();
            Ok(reservas
                .values()
                .filter(|r| {
                    r.sala_id() == sala_id
                        && r.esta_activa()
                        && r.fecha_inicio() < fin
                        && r.fecha_fin() > inicio
                })
                .cloned()
                .collect())
        }

        async fn actualizar(
            &self,
            reserva: &Reserva,
        ) -> Result<(), reservas_domain::ReservaError> {
            let mut reservas = self.reservas.lock().unwrap();
            if reservas.contains_key(reserva.id()) {
                reservas.insert(reserva.id().to_string(), reserva.clone());
                Ok(())
            } else {
                Err(reservas_domain::ReservaError::NoEncontrada)
            }
        }

        async fn eliminar(&self, id: &str) -> Result<(), reservas_domain::ReservaError> {
            let mut reservas = self.reservas.lock().unwrap();
            if reservas.remove(id).is_some() {
                Ok(())
            } else {
                Err(reservas_domain::ReservaError::NoEncontrada)
            }
        }
    }

    #[tokio::test]
    async fn test_crear_reserva_grpc() {
        let repo = MockReservaRepository::new();
        let service = ReservaServiceImpl::new(repo);
        let grpc_server = ReservaGrpcServer::new(service);

        let ahora = Utc::now();
        let inicio = ahora + Duration::hours(1);
        let fin = inicio + Duration::hours(2);

        let request = Request::new(CrearReservaRequest {
            sala_id: "sala1".to_string(),
            usuario_id: "usuario1".to_string(),
            fecha_inicio: inicio.to_rfc3339(),
            fecha_fin: fin.to_rfc3339(),
        });

        let response = grpc_server.crear_reserva(request).await.unwrap();
        let reserva = response.into_inner().reserva.unwrap();

        assert_eq!(reserva.sala_id, "sala1");
        assert_eq!(reserva.usuario_id, "usuario1");
        assert_eq!(reserva.estado, ProtoEstadoReserva::Activa as i32);
    }

    #[tokio::test]
    async fn test_listar_reservas_grpc() {
        let repo = MockReservaRepository::new();
        let service = ReservaServiceImpl::new(repo);
        let grpc_server = ReservaGrpcServer::new(service);

        let ahora = Utc::now();

        // Crear primera reserva
        let request1 = Request::new(CrearReservaRequest {
            sala_id: "sala1".to_string(),
            usuario_id: "usuario1".to_string(),
            fecha_inicio: (ahora + Duration::hours(1)).to_rfc3339(),
            fecha_fin: (ahora + Duration::hours(2)).to_rfc3339(),
        });
        grpc_server.crear_reserva(request1).await.unwrap();

        // Crear segunda reserva
        let request2 = Request::new(CrearReservaRequest {
            sala_id: "sala2".to_string(),
            usuario_id: "usuario2".to_string(),
            fecha_inicio: (ahora + Duration::hours(3)).to_rfc3339(),
            fecha_fin: (ahora + Duration::hours(4)).to_rfc3339(),
        });
        grpc_server.crear_reserva(request2).await.unwrap();

        // Listar todas
        let request = Request::new(ListarReservasRequest {});
        let response = grpc_server.listar_reservas(request).await.unwrap();
        let reservas = response.into_inner().reservas;

        assert_eq!(reservas.len(), 2);
    }

    #[tokio::test]
    async fn test_cancelar_reserva_grpc() {
        let repo = MockReservaRepository::new();
        let service = ReservaServiceImpl::new(repo);
        let grpc_server = ReservaGrpcServer::new(service);

        let ahora = Utc::now();

        // Crear reserva
        let request = Request::new(CrearReservaRequest {
            sala_id: "sala1".to_string(),
            usuario_id: "usuario1".to_string(),
            fecha_inicio: (ahora + Duration::hours(1)).to_rfc3339(),
            fecha_fin: (ahora + Duration::hours(2)).to_rfc3339(),
        });
        let response = grpc_server.crear_reserva(request).await.unwrap();
        let id = response.into_inner().reserva.unwrap().id;

        // Cancelar reserva
        let request = Request::new(CancelarReservaRequest { id });
        let response = grpc_server.cancelar_reserva(request).await.unwrap();
        let reserva = response.into_inner().reserva.unwrap();

        assert_eq!(reserva.estado, ProtoEstadoReserva::Cancelada as i32);
    }

    #[tokio::test]
    async fn test_verificar_disponibilidad_grpc() {
        let repo = MockReservaRepository::new();
        let service = ReservaServiceImpl::new(repo);
        let grpc_server = ReservaGrpcServer::new(service);

        let ahora = Utc::now();

        // Crear reserva
        let request = Request::new(CrearReservaRequest {
            sala_id: "sala1".to_string(),
            usuario_id: "usuario1".to_string(),
            fecha_inicio: (ahora + Duration::hours(1)).to_rfc3339(),
            fecha_fin: (ahora + Duration::hours(3)).to_rfc3339(),
        });
        grpc_server.crear_reserva(request).await.unwrap();

        // Verificar horario ocupado
        let request = Request::new(VerificarDisponibilidadRequest {
            sala_id: "sala1".to_string(),
            fecha_inicio: (ahora + Duration::hours(2)).to_rfc3339(),
            fecha_fin: (ahora + Duration::hours(4)).to_rfc3339(),
        });
        let response = grpc_server.verificar_disponibilidad(request).await.unwrap();
        assert!(!response.into_inner().disponible);

        // Verificar horario libre
        let request = Request::new(VerificarDisponibilidadRequest {
            sala_id: "sala1".to_string(),
            fecha_inicio: (ahora + Duration::hours(5)).to_rfc3339(),
            fecha_fin: (ahora + Duration::hours(6)).to_rfc3339(),
        });
        let response = grpc_server.verificar_disponibilidad(request).await.unwrap();
        assert!(response.into_inner().disponible);
    }
}
