use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reservas_domain::{EstadoReserva, Reserva, ReservaError};
use salas_application::SalaRepository;
use usuarios_application::UsuarioRepository;

use crate::repository::ReservaRepository;

/// Trait del servicio de reservas (casos de uso)
#[async_trait]
pub trait ReservaService: Send + Sync {
    /// Crea una nueva reserva
    async fn crear_reserva(
        &self,
        sala_id: String,
        usuario_id: String,
        fecha_inicio: DateTime<Utc>,
        fecha_fin: DateTime<Utc>,
    ) -> Result<Reserva, ReservaError>;

    /// Obtiene una reserva por su ID
    async fn obtener_reserva(&self, id: &str) -> Result<Option<Reserva>, ReservaError>;

    /// Lista todas las reservas
    async fn listar_reservas(&self) -> Result<Vec<Reserva>, ReservaError>;

    /// Lista reservas de una sala específica
    async fn listar_reservas_por_sala(&self, sala_id: &str) -> Result<Vec<Reserva>, ReservaError>;

    /// Lista reservas de un usuario específico
    async fn listar_reservas_por_usuario(
        &self,
        usuario_id: &str,
    ) -> Result<Vec<Reserva>, ReservaError>;

    /// Cancela una reserva existente
    async fn cancelar_reserva(&self, id: &str) -> Result<Reserva, ReservaError>;

    /// Completa una reserva (marca como finalizada)
    async fn completar_reserva(&self, id: &str) -> Result<Reserva, ReservaError>;

    /// Verifica disponibilidad de una sala en un rango de fechas
    async fn verificar_disponibilidad(
        &self,
        sala_id: &str,
        fecha_inicio: DateTime<Utc>,
        fecha_fin: DateTime<Utc>,
    ) -> Result<bool, ReservaError>;
}

/// Implementación del servicio de reservas
pub struct ReservaServiceImpl<R: ReservaRepository, S: SalaRepository, U: UsuarioRepository> {
    repository: R,
    sala_repository: S,
    usuario_repository: U,
}

impl<R: ReservaRepository, S: SalaRepository, U: UsuarioRepository> ReservaServiceImpl<R, S, U> {
    pub fn new(repository: R, sala_repository: S, usuario_repository: U) -> Self {
        Self {
            repository,
            sala_repository,
            usuario_repository,
        }
    }
}

#[async_trait]
impl<R: ReservaRepository, S: SalaRepository, U: UsuarioRepository> ReservaService
    for ReservaServiceImpl<R, S, U>
{
    async fn crear_reserva(
        &self,
        sala_id: String,
        usuario_id: String,
        fecha_inicio: DateTime<Utc>,
        fecha_fin: DateTime<Utc>,
    ) -> Result<Reserva, ReservaError> {
        // Validar que la sala existe
        let sala = self
            .sala_repository
            .obtener(&sala_id)
            .await
            .map_err(|e| ReservaError::ErrorRepositorio(format!("Error al verificar sala: {}", e)))?
            .ok_or_else(|| ReservaError::Validacion(vec!["La sala no existe".to_string()]))?;

        // Validar que la sala está activa
        if !sala.esta_activa() {
            return Err(ReservaError::Validacion(vec![
                "La sala no está activa".to_string()
            ]));
        }

        // Validar que el usuario existe
        self.usuario_repository
            .obtener(&usuario_id)
            .await
            .map_err(|e| {
                ReservaError::ErrorRepositorio(format!("Error al verificar usuario: {}", e))
            })?
            .ok_or_else(|| ReservaError::Validacion(vec!["El usuario no existe".to_string()]))?;

        // Crear la reserva (valida fechas, duración, etc.)
        let reserva = Reserva::new(sala_id.clone(), usuario_id, fecha_inicio, fecha_fin)?;

        // Verificar disponibilidad (no debe solaparse con otras reservas activas)
        let disponible = self
            .verificar_disponibilidad(&sala_id, fecha_inicio, fecha_fin)
            .await?;

        if !disponible {
            return Err(ReservaError::Validacion(vec![
                "La sala no está disponible en el horario solicitado".to_string(),
            ]));
        }

        // Guardar en el repositorio
        self.repository.guardar(&reserva).await?;

        Ok(reserva)
    }

    async fn obtener_reserva(&self, id: &str) -> Result<Option<Reserva>, ReservaError> {
        self.repository.obtener(id).await
    }

    async fn listar_reservas(&self) -> Result<Vec<Reserva>, ReservaError> {
        self.repository.listar().await
    }

    async fn listar_reservas_por_sala(&self, sala_id: &str) -> Result<Vec<Reserva>, ReservaError> {
        self.repository.listar_por_sala(sala_id).await
    }

    async fn listar_reservas_por_usuario(
        &self,
        usuario_id: &str,
    ) -> Result<Vec<Reserva>, ReservaError> {
        self.repository.listar_por_usuario(usuario_id).await
    }

    async fn cancelar_reserva(&self, id: &str) -> Result<Reserva, ReservaError> {
        let mut reserva = self
            .repository
            .obtener(id)
            .await?
            .ok_or(ReservaError::NoEncontrada)?;

        // Solo se pueden cancelar reservas activas
        if !reserva.esta_activa() {
            return Err(ReservaError::Validacion(vec![
                "Solo se pueden cancelar reservas activas".to_string(),
            ]));
        }

        reserva.cancelar();
        self.repository.actualizar(&reserva).await?;

        Ok(reserva)
    }

    async fn completar_reserva(&self, id: &str) -> Result<Reserva, ReservaError> {
        let mut reserva = self
            .repository
            .obtener(id)
            .await?
            .ok_or(ReservaError::NoEncontrada)?;

        // Solo se pueden completar reservas activas
        if !reserva.esta_activa() {
            return Err(ReservaError::Validacion(vec![
                "Solo se pueden completar reservas activas".to_string(),
            ]));
        }

        reserva.completar();
        self.repository.actualizar(&reserva).await?;

        Ok(reserva)
    }

    async fn verificar_disponibilidad(
        &self,
        sala_id: &str,
        fecha_inicio: DateTime<Utc>,
        fecha_fin: DateTime<Utc>,
    ) -> Result<bool, ReservaError> {
        // Obtener todas las reservas activas de la sala en el rango de fechas
        let reservas = self
            .repository
            .listar_por_sala_y_rango(sala_id, fecha_inicio, fecha_fin)
            .await?;

        // Crear una reserva temporal para verificar solapamientos
        // Usamos from_existing para evitar validaciones de fecha pasada
        let reserva_temporal = Reserva::from_existing(
            "temp".to_string(),
            sala_id.to_string(),
            "temp_user".to_string(),
            fecha_inicio,
            fecha_fin,
            EstadoReserva::Activa,
            Utc::now(),
        );

        // Verificar si hay solapamiento con alguna reserva existente
        let hay_conflicto = reservas
            .iter()
            .any(|r| reserva_temporal.se_solapa_con(r));

        Ok(!hay_conflicto)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Mock repositories para testing
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

    struct MockSalaRepository;
    struct MockUsuarioRepository;

    #[async_trait]
    impl salas_application::SalaRepository for MockSalaRepository {
        async fn guardar(&self, _sala: &salas_domain::Sala) -> Result<(), salas_domain::SalaError> {
            Ok(())
        }

        async fn obtener(&self, _id: &str) -> Result<Option<salas_domain::Sala>, salas_domain::SalaError> {
            // Siempre devuelve una sala válida para los tests
            Ok(Some(
                salas_domain::Sala::new("sala1".to_string(), "Sala Test".to_string(), 10).unwrap(),
            ))
        }

        async fn listar(&self) -> Result<Vec<salas_domain::Sala>, salas_domain::SalaError> {
            Ok(vec![])
        }

        async fn actualizar(&self, _sala: &salas_domain::Sala) -> Result<(), salas_domain::SalaError> {
            Ok(())
        }
    }

    #[async_trait]
    impl usuarios_application::UsuarioRepository for MockUsuarioRepository {
        async fn guardar(&self, _usuario: &usuarios_domain::Usuario) -> Result<(), usuarios_domain::UsuarioError> {
            Ok(())
        }

        async fn obtener(&self, _id: &str) -> Result<Option<usuarios_domain::Usuario>, usuarios_domain::UsuarioError> {
            // Siempre devuelve un usuario válido para los tests
            use usuarios_domain::{Usuario, Rol};
            let now = chrono::Utc::now();
            Ok(Some(Usuario::with_id(
                "usuario1".to_string(),
                "Test User".to_string(),
                "test@example.com".to_string(),
                "hashed_password".to_string(),
                Rol::Usuario,
                now,
                now,
                true,
            )?))
        }

        async fn obtener_por_email(&self, _email: &str) -> Result<Option<usuarios_domain::Usuario>, usuarios_domain::UsuarioError> {
            Ok(None)
        }

        async fn listar(&self) -> Result<Vec<usuarios_domain::Usuario>, usuarios_domain::UsuarioError> {
            Ok(vec![])
        }

        async fn actualizar(&self, _usuario: &usuarios_domain::Usuario) -> Result<(), usuarios_domain::UsuarioError> {
            Ok(())
        }

        async fn eliminar(&self, _id: &str) -> Result<(), usuarios_domain::UsuarioError> {
            Ok(())
        }

        async fn existe_email(&self, _email: &str) -> Result<bool, usuarios_domain::UsuarioError> {
            Ok(false)
        }
    }

    #[async_trait]
    impl ReservaRepository for MockReservaRepository {
        async fn guardar(&self, reserva: &Reserva) -> Result<(), ReservaError> {
            let mut reservas = self.reservas.lock().unwrap();
            reservas.insert(reserva.id().to_string(), reserva.clone());
            Ok(())
        }

        async fn obtener(&self, id: &str) -> Result<Option<Reserva>, ReservaError> {
            let reservas = self.reservas.lock().unwrap();
            Ok(reservas.get(id).cloned())
        }

        async fn listar(&self) -> Result<Vec<Reserva>, ReservaError> {
            let reservas = self.reservas.lock().unwrap();
            Ok(reservas.values().cloned().collect())
        }

        async fn listar_por_sala(&self, sala_id: &str) -> Result<Vec<Reserva>, ReservaError> {
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
        ) -> Result<Vec<Reserva>, ReservaError> {
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
        ) -> Result<Vec<Reserva>, ReservaError> {
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

        async fn actualizar(&self, reserva: &Reserva) -> Result<(), ReservaError> {
            let mut reservas = self.reservas.lock().unwrap();
            if reservas.contains_key(reserva.id()) {
                reservas.insert(reserva.id().to_string(), reserva.clone());
                Ok(())
            } else {
                Err(ReservaError::NoEncontrada)
            }
        }

        async fn eliminar(&self, id: &str) -> Result<(), ReservaError> {
            let mut reservas = self.reservas.lock().unwrap();
            if reservas.remove(id).is_some() {
                Ok(())
            } else {
                Err(ReservaError::NoEncontrada)
            }
        }
    }

    #[tokio::test]
    async fn test_crear_reserva_valida() {
        let repo = MockReservaRepository::new();
        let service = ReservaServiceImpl::new(repo, MockSalaRepository, MockUsuarioRepository);

        let ahora = Utc::now();
        let inicio = ahora + Duration::hours(1);
        let fin = inicio + Duration::hours(2);

        let resultado = service
            .crear_reserva("sala1".into(), "usuario1".into(), inicio, fin)
            .await;

        assert!(resultado.is_ok());
        let reserva = resultado.unwrap();
        assert_eq!(reserva.sala_id(), "sala1");
        assert_eq!(reserva.usuario_id(), "usuario1");
        assert!(reserva.esta_activa());
    }

    #[tokio::test]
    async fn test_crear_reserva_con_conflicto() {
        let repo = MockReservaRepository::new();
        let service = ReservaServiceImpl::new(repo, MockSalaRepository, MockUsuarioRepository);

        let ahora = Utc::now();
        let inicio1 = ahora + Duration::hours(1);
        let fin1 = inicio1 + Duration::hours(2);

        // Crear primera reserva
        let _ = service
            .crear_reserva("sala1".into(), "usuario1".into(), inicio1, fin1)
            .await
            .unwrap();

        // Intentar crear segunda reserva que se solapa
        let inicio2 = ahora + Duration::hours(2);
        let fin2 = inicio2 + Duration::hours(2);

        let resultado = service
            .crear_reserva("sala1".into(), "usuario2".into(), inicio2, fin2)
            .await;

        assert!(resultado.is_err());
        match resultado.unwrap_err() {
            ReservaError::Validacion(msgs) => {
                assert!(msgs
                    .iter()
                    .any(|m| m.contains("no está disponible")));
            }
            _ => panic!("Se esperaba error de validación"),
        }
    }

    #[tokio::test]
    async fn test_crear_reservas_sin_conflicto() {
        let repo = MockReservaRepository::new();
        let service = ReservaServiceImpl::new(repo, MockSalaRepository, MockUsuarioRepository);

        let ahora = Utc::now();
        let inicio1 = ahora + Duration::hours(1);
        let fin1 = inicio1 + Duration::hours(1);

        // Crear primera reserva
        let _ = service
            .crear_reserva("sala1".into(), "usuario1".into(), inicio1, fin1)
            .await
            .unwrap();

        // Crear segunda reserva que NO se solapa
        let inicio2 = ahora + Duration::hours(3);
        let fin2 = inicio2 + Duration::hours(1);

        let resultado = service
            .crear_reserva("sala1".into(), "usuario2".into(), inicio2, fin2)
            .await;

        assert!(resultado.is_ok());
    }

    #[tokio::test]
    async fn test_cancelar_reserva() {
        let repo = MockReservaRepository::new();
        let service = ReservaServiceImpl::new(repo, MockSalaRepository, MockUsuarioRepository);

        let ahora = Utc::now();
        let inicio = ahora + Duration::hours(1);
        let fin = inicio + Duration::hours(2);

        let reserva = service
            .crear_reserva("sala1".into(), "usuario1".into(), inicio, fin)
            .await
            .unwrap();

        let id = reserva.id().to_string();

        let resultado = service.cancelar_reserva(&id).await;
        assert!(resultado.is_ok());

        let reserva_cancelada = resultado.unwrap();
        assert!(!reserva_cancelada.esta_activa());
        assert_eq!(reserva_cancelada.estado(), &EstadoReserva::Cancelada);
    }

    #[tokio::test]
    async fn test_completar_reserva() {
        let repo = MockReservaRepository::new();
        let service = ReservaServiceImpl::new(repo, MockSalaRepository, MockUsuarioRepository);

        let ahora = Utc::now();
        let inicio = ahora + Duration::hours(1);
        let fin = inicio + Duration::hours(2);

        let reserva = service
            .crear_reserva("sala1".into(), "usuario1".into(), inicio, fin)
            .await
            .unwrap();

        let id = reserva.id().to_string();

        let resultado = service.completar_reserva(&id).await;
        assert!(resultado.is_ok());

        let reserva_completada = resultado.unwrap();
        assert!(!reserva_completada.esta_activa());
        assert_eq!(reserva_completada.estado(), &EstadoReserva::Completada);
    }

    #[tokio::test]
    async fn test_no_cancelar_reserva_ya_cancelada() {
        let repo = MockReservaRepository::new();
        let service = ReservaServiceImpl::new(repo, MockSalaRepository, MockUsuarioRepository);

        let ahora = Utc::now();
        let inicio = ahora + Duration::hours(1);
        let fin = inicio + Duration::hours(2);

        let reserva = service
            .crear_reserva("sala1".into(), "usuario1".into(), inicio, fin)
            .await
            .unwrap();

        let id = reserva.id().to_string();

        // Cancelar por primera vez
        service.cancelar_reserva(&id).await.unwrap();

        // Intentar cancelar de nuevo
        let resultado = service.cancelar_reserva(&id).await;
        assert!(resultado.is_err());
    }

    #[tokio::test]
    async fn test_listar_reservas_por_sala() {
        let repo = MockReservaRepository::new();
        let service = ReservaServiceImpl::new(repo, MockSalaRepository, MockUsuarioRepository);

        let ahora = Utc::now();

        // Crear reservas para sala1
        let _ = service
            .crear_reserva(
                "sala1".into(),
                "usuario1".into(),
                ahora + Duration::hours(1),
                ahora + Duration::hours(2),
            )
            .await
            .unwrap();

        let _ = service
            .crear_reserva(
                "sala1".into(),
                "usuario2".into(),
                ahora + Duration::hours(3),
                ahora + Duration::hours(4),
            )
            .await
            .unwrap();

        // Crear reserva para sala2
        let _ = service
            .crear_reserva(
                "sala2".into(),
                "usuario1".into(),
                ahora + Duration::hours(1),
                ahora + Duration::hours(2),
            )
            .await
            .unwrap();

        let reservas_sala1 = service.listar_reservas_por_sala("sala1").await.unwrap();
        assert_eq!(reservas_sala1.len(), 2);

        let reservas_sala2 = service.listar_reservas_por_sala("sala2").await.unwrap();
        assert_eq!(reservas_sala2.len(), 1);
    }

    #[tokio::test]
    async fn test_listar_reservas_por_usuario() {
        let repo = MockReservaRepository::new();
        let service = ReservaServiceImpl::new(repo, MockSalaRepository, MockUsuarioRepository);

        let ahora = Utc::now();

        // Crear reservas para usuario1
        let _ = service
            .crear_reserva(
                "sala1".into(),
                "usuario1".into(),
                ahora + Duration::hours(1),
                ahora + Duration::hours(2),
            )
            .await
            .unwrap();

        let _ = service
            .crear_reserva(
                "sala2".into(),
                "usuario1".into(),
                ahora + Duration::hours(3),
                ahora + Duration::hours(4),
            )
            .await
            .unwrap();

        // Crear reserva para usuario2
        let _ = service
            .crear_reserva(
                "sala1".into(),
                "usuario2".into(),
                ahora + Duration::hours(5),
                ahora + Duration::hours(6),
            )
            .await
            .unwrap();

        let reservas_usuario1 = service
            .listar_reservas_por_usuario("usuario1")
            .await
            .unwrap();
        assert_eq!(reservas_usuario1.len(), 2);

        let reservas_usuario2 = service
            .listar_reservas_por_usuario("usuario2")
            .await
            .unwrap();
        assert_eq!(reservas_usuario2.len(), 1);
    }

    #[tokio::test]
    async fn test_verificar_disponibilidad() {
        let repo = MockReservaRepository::new();
        let service = ReservaServiceImpl::new(repo, MockSalaRepository, MockUsuarioRepository);

        let ahora = Utc::now();
        let inicio1 = ahora + Duration::hours(1);
        let fin1 = inicio1 + Duration::hours(2);

        // Crear reserva
        let _ = service
            .crear_reserva("sala1".into(), "usuario1".into(), inicio1, fin1)
            .await
            .unwrap();

        // Verificar disponibilidad en horario ocupado
        let disponible = service
            .verificar_disponibilidad("sala1", inicio1 + Duration::hours(1), fin1 + Duration::hours(1))
            .await
            .unwrap();
        assert!(!disponible);

        // Verificar disponibilidad en horario libre
        let disponible = service
            .verificar_disponibilidad(
                "sala1",
                ahora + Duration::hours(4),
                ahora + Duration::hours(5),
            )
            .await
            .unwrap();
        assert!(disponible);
    }
}
