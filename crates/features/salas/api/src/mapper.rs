use crate::dtos::SalaResponse;
use salas_domain::Sala;

pub struct SalaMapper;

impl SalaMapper {
    pub fn to_response(sala: &Sala) -> SalaResponse {
        SalaResponse {
            id: sala.id.clone(),
            nombre: sala.nombre.clone(),
            capacidad: sala.capacidad,
            activa: sala.activa,
        }
    }
}