use crate::dtos::SalaResponse;
use salas_domain::Sala;

pub struct SalaMapper;

impl From<&Sala> for SalaResponse {
    fn from(sala: &Sala) -> Self {
        SalaResponse {
            id: sala.id.clone(),
            nombre: sala.nombre.clone(),
            capacidad: sala.capacidad,
            activa: sala.activa,
        }
    }
}

impl From<Sala> for SalaResponse {
    fn from(sala: Sala) -> Self {
        SalaResponse {
            id: sala.id,
            nombre: sala.nombre,
            capacidad: sala.capacidad,
            activa: sala.activa,
        }
    }
}
