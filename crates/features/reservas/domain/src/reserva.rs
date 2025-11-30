use crate::error::ReservaError;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Estado de una reserva
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EstadoReserva {
    Activa,
    Cancelada,
    Completada,
}

/// Entidad Reserva: representa la reserva de una sala por un usuario en un período de tiempo
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reserva {
    pub id: String,
    pub sala_id: String,
    pub usuario_id: String,
    pub fecha_inicio: DateTime<Utc>,
    pub fecha_fin: DateTime<Utc>,
    pub estado: EstadoReserva,
    pub created_at: DateTime<Utc>,
}

impl Reserva {
    /// Crea una nueva reserva con validaciones
    pub fn new(
        sala_id: String,
        usuario_id: String,
        fecha_inicio: DateTime<Utc>,
        fecha_fin: DateTime<Utc>,
    ) -> Result<Self, ReservaError> {
        let mut errores: Vec<String> = Vec::new();

        // Validar que los IDs no estén vacíos
        if sala_id.trim().is_empty() {
            errores.push("El ID de sala no puede estar vacío".to_string());
        }

        if usuario_id.trim().is_empty() {
            errores.push("El ID de usuario no puede estar vacío".to_string());
        }

        // Validar fechas
        let ahora = Utc::now();

        if fecha_inicio < ahora {
            errores.push("La fecha de inicio no puede ser en el pasado".to_string());
        }

        if fecha_fin < ahora {
            errores.push("La fecha de fin no puede ser en el pasado".to_string());
        }

        if fecha_fin <= fecha_inicio {
            errores.push("La fecha de fin debe ser posterior a la fecha de inicio".to_string());
        }

        // Validar duración (mínimo 15 minutos, máximo 8 horas)
        let duracion = fecha_fin - fecha_inicio;
        let min_duracion = Duration::minutes(15);
        let max_duracion = Duration::hours(8);

        if duracion < min_duracion || duracion > max_duracion {
            errores.push(
                "La duración de la reserva debe ser entre 15 minutos y 8 horas".to_string(),
            );
        }

        // Si hay errores, devolver todos
        if !errores.is_empty() {
            return Err(ReservaError::Validacion(errores));
        }

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            sala_id,
            usuario_id,
            fecha_inicio,
            fecha_fin,
            estado: EstadoReserva::Activa,
            created_at: Utc::now(),
        })
    }

    /// Crea una reserva desde datos existentes (para cargar desde repositorio)
    pub fn from_existing(
        id: String,
        sala_id: String,
        usuario_id: String,
        fecha_inicio: DateTime<Utc>,
        fecha_fin: DateTime<Utc>,
        estado: EstadoReserva,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            sala_id,
            usuario_id,
            fecha_inicio,
            fecha_fin,
            estado,
            created_at,
        }
    }

    // Getters
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn sala_id(&self) -> &str {
        &self.sala_id
    }

    pub fn usuario_id(&self) -> &str {
        &self.usuario_id
    }

    pub fn fecha_inicio(&self) -> DateTime<Utc> {
        self.fecha_inicio
    }

    pub fn fecha_fin(&self) -> DateTime<Utc> {
        self.fecha_fin
    }

    pub fn estado(&self) -> &EstadoReserva {
        &self.estado
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    // Métodos de negocio
    pub fn esta_activa(&self) -> bool {
        matches!(self.estado, EstadoReserva::Activa)
    }

    pub fn cancelar(&mut self) {
        self.estado = EstadoReserva::Cancelada;
    }

    pub fn completar(&mut self) {
        self.estado = EstadoReserva::Completada;
    }

    /// Verifica si esta reserva se solapa con otra
    pub fn se_solapa_con(&self, otra: &Reserva) -> bool {
        // Solo verificar solapamiento si ambas reservas son para la misma sala
        if self.sala_id != otra.sala_id {
            return false;
        }

        // Solo verificar solapamiento si ambas reservas están activas
        if !self.esta_activa() || !otra.esta_activa() {
            return false;
        }

        // Verificar solapamiento de fechas
        self.fecha_inicio < otra.fecha_fin && otra.fecha_inicio < self.fecha_fin
    }

    /// Obtiene la duración de la reserva en minutos
    pub fn duracion_minutos(&self) -> i64 {
        (self.fecha_fin - self.fecha_inicio).num_minutes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn extraer_errores(result: Result<Reserva, ReservaError>) -> Result<Vec<String>, String> {
        match result {
            Err(ReservaError::Validacion(errores)) => Ok(errores),
            Err(e) => Err(format!("Se esperaba Validacion, pero se obtuvo: {:?}", e)),
            Ok(r) => Err(format!("Se esperaba error, pero se obtuvo Ok({:?})", r)),
        }
    }

    fn assert_contiene_error(errores: &[String], claves: &[&str]) -> Result<(), String> {
        if errores.is_empty() {
            return Err("La lista de errores no debería estar vacía".into());
        }

        if errores.iter().any(|e| claves.iter().any(|k| e.contains(k))) {
            Ok(())
        } else {
            Err(format!(
                "Ningún mensaje contiene {:?}. Errores: {:?}",
                claves, errores
            ))
        }
    }

    #[test]
    fn crear_reserva_valida() -> Result<(), String> {
        let ahora = Utc::now();
        let inicio = ahora + Duration::hours(1);
        let fin = inicio + Duration::hours(2);

        let reserva = Reserva::new("sala1".into(), "usuario1".into(), inicio, fin)
            .map_err(|e| format!("No debería fallar: {:?}", e))?;

        assert_eq!(reserva.sala_id(), "sala1");
        assert_eq!(reserva.usuario_id(), "usuario1");
        assert_eq!(reserva.fecha_inicio(), inicio);
        assert_eq!(reserva.fecha_fin(), fin);
        assert!(reserva.esta_activa());
        assert_eq!(reserva.duracion_minutos(), 120);

        Ok(())
    }

    #[test]
    fn crear_reserva_con_sala_id_vacio() -> Result<(), String> {
        let ahora = Utc::now();
        let inicio = ahora + Duration::hours(1);
        let fin = inicio + Duration::hours(2);

        let errores = extraer_errores(Reserva::new("".into(), "usuario1".into(), inicio, fin))?;
        assert_contiene_error(&errores, &["sala", "vacío"])
    }

    #[test]
    fn crear_reserva_con_usuario_id_vacio() -> Result<(), String> {
        let ahora = Utc::now();
        let inicio = ahora + Duration::hours(1);
        let fin = inicio + Duration::hours(2);

        let errores = extraer_errores(Reserva::new("sala1".into(), "".into(), inicio, fin))?;
        assert_contiene_error(&errores, &["usuario", "vacío"])
    }

    #[test]
    fn crear_reserva_con_fecha_inicio_en_pasado() -> Result<(), String> {
        let ahora = Utc::now();
        let inicio = ahora - Duration::hours(1);
        let fin = ahora + Duration::hours(1);

        let errores = extraer_errores(Reserva::new("sala1".into(), "usuario1".into(), inicio, fin))?;
        assert_contiene_error(&errores, &["inicio", "pasado"])
    }

    #[test]
    fn crear_reserva_con_fecha_fin_anterior_a_inicio() -> Result<(), String> {
        let ahora = Utc::now();
        let inicio = ahora + Duration::hours(2);
        let fin = ahora + Duration::hours(1);

        let errores = extraer_errores(Reserva::new("sala1".into(), "usuario1".into(), inicio, fin))?;
        assert_contiene_error(&errores, &["fin", "posterior"])
    }

    #[test]
    fn crear_reserva_con_duracion_muy_corta() -> Result<(), String> {
        let ahora = Utc::now();
        let inicio = ahora + Duration::hours(1);
        let fin = inicio + Duration::minutes(10);

        let errores = extraer_errores(Reserva::new("sala1".into(), "usuario1".into(), inicio, fin))?;
        assert_contiene_error(&errores, &["duración", "15 minutos", "8 horas"])
    }

    #[test]
    fn crear_reserva_con_duracion_muy_larga() -> Result<(), String> {
        let ahora = Utc::now();
        let inicio = ahora + Duration::hours(1);
        let fin = inicio + Duration::hours(10);

        let errores = extraer_errores(Reserva::new("sala1".into(), "usuario1".into(), inicio, fin))?;
        assert_contiene_error(&errores, &["duración", "15 minutos", "8 horas"])
    }

    #[test]
    fn cancelar_reserva() -> Result<(), String> {
        let ahora = Utc::now();
        let inicio = ahora + Duration::hours(1);
        let fin = inicio + Duration::hours(2);

        let mut reserva = Reserva::new("sala1".into(), "usuario1".into(), inicio, fin)
            .map_err(|e| format!("No debería fallar: {:?}", e))?;

        assert!(reserva.esta_activa());

        reserva.cancelar();
        assert!(!reserva.esta_activa());
        assert_eq!(reserva.estado(), &EstadoReserva::Cancelada);

        Ok(())
    }

    #[test]
    fn completar_reserva() -> Result<(), String> {
        let ahora = Utc::now();
        let inicio = ahora + Duration::hours(1);
        let fin = inicio + Duration::hours(2);

        let mut reserva = Reserva::new("sala1".into(), "usuario1".into(), inicio, fin)
            .map_err(|e| format!("No debería fallar: {:?}", e))?;

        reserva.completar();
        assert!(!reserva.esta_activa());
        assert_eq!(reserva.estado(), &EstadoReserva::Completada);

        Ok(())
    }

    #[test]
    fn detectar_solapamiento_entre_reservas() -> Result<(), String> {
        let ahora = Utc::now();
        let inicio1 = ahora + Duration::hours(1);
        let fin1 = inicio1 + Duration::hours(2);

        let inicio2 = ahora + Duration::hours(2);
        let fin2 = inicio2 + Duration::hours(2);

        let reserva1 = Reserva::new("sala1".into(), "usuario1".into(), inicio1, fin1)
            .map_err(|e| format!("No debería fallar: {:?}", e))?;

        let reserva2 = Reserva::new("sala1".into(), "usuario2".into(), inicio2, fin2)
            .map_err(|e| format!("No debería fallar: {:?}", e))?;

        assert!(reserva1.se_solapa_con(&reserva2));
        assert!(reserva2.se_solapa_con(&reserva1));

        Ok(())
    }

    #[test]
    fn no_detectar_solapamiento_sin_conflicto() -> Result<(), String> {
        let ahora = Utc::now();
        let inicio1 = ahora + Duration::hours(1);
        let fin1 = inicio1 + Duration::hours(1);

        let inicio2 = ahora + Duration::hours(3);
        let fin2 = inicio2 + Duration::hours(1);

        let reserva1 = Reserva::new("sala1".into(), "usuario1".into(), inicio1, fin1)
            .map_err(|e| format!("No debería fallar: {:?}", e))?;

        let reserva2 = Reserva::new("sala1".into(), "usuario2".into(), inicio2, fin2)
            .map_err(|e| format!("No debería fallar: {:?}", e))?;

        assert!(!reserva1.se_solapa_con(&reserva2));
        assert!(!reserva2.se_solapa_con(&reserva1));

        Ok(())
    }

    #[test]
    fn no_detectar_solapamiento_entre_salas_diferentes() -> Result<(), String> {
        let ahora = Utc::now();
        let inicio1 = ahora + Duration::hours(1);
        let fin1 = inicio1 + Duration::hours(2);

        let inicio2 = ahora + Duration::hours(2);
        let fin2 = inicio2 + Duration::hours(2);

        let reserva1 = Reserva::new("sala1".into(), "usuario1".into(), inicio1, fin1)
            .map_err(|e| format!("No debería fallar: {:?}", e))?;

        let reserva2 = Reserva::new("sala2".into(), "usuario2".into(), inicio2, fin2)
            .map_err(|e| format!("No debería fallar: {:?}", e))?;

        assert!(!reserva1.se_solapa_con(&reserva2));
        assert!(!reserva2.se_solapa_con(&reserva1));

        Ok(())
    }

    #[test]
    fn no_detectar_solapamiento_con_reservas_canceladas() -> Result<(), String> {
        let ahora = Utc::now();
        let inicio1 = ahora + Duration::hours(1);
        let fin1 = inicio1 + Duration::hours(2);

        let inicio2 = ahora + Duration::hours(2);
        let fin2 = inicio2 + Duration::hours(2);

        let mut reserva1 = Reserva::new("sala1".into(), "usuario1".into(), inicio1, fin1)
            .map_err(|e| format!("No debería fallar: {:?}", e))?;

        let reserva2 = Reserva::new("sala1".into(), "usuario2".into(), inicio2, fin2)
            .map_err(|e| format!("No debería fallar: {:?}", e))?;

        reserva1.cancelar();

        assert!(!reserva1.se_solapa_con(&reserva2));
        assert!(!reserva2.se_solapa_con(&reserva1));

        Ok(())
    }
}
