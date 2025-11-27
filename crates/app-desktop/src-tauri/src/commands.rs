use crate::SharedSalaService;
use salas_domain::Sala;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct SalaDto {
    pub id: String,
    pub nombre: String,
    pub capacidad: u32,
    pub activa: bool,
}

impl From<Sala> for SalaDto {
    fn from(sala: Sala) -> Self {
        Self {
            id: sala.id,
            nombre: sala.nombre,
            capacidad: sala.capacidad,
            activa: sala.activa,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrearSalaRequest {
    pub nombre: String,
    pub capacidad: u32,
}

#[tauri::command]
pub async fn crear_sala(
    request: CrearSalaRequest,
    service: State<'_, SharedSalaService>,
) -> Result<SalaDto, String> {
    service
        .crear_sala(request.nombre, request.capacidad)
        .await
        .map(|sala| sala.into())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn listar_salas(service: State<'_, SharedSalaService>) -> Result<Vec<SalaDto>, String> {
    service
        .listar_salas()
        .await
        .map(|salas| salas.into_iter().map(|s| s.into()).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn obtener_sala(
    id: String,
    service: State<'_, SharedSalaService>,
) -> Result<Option<SalaDto>, String> {
    service
        .obtener_sala(&id)
        .await
        .map(|opt_sala| opt_sala.map(|s| s.into()))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn activar_sala(
    id: String,
    service: State<'_, SharedSalaService>,
) -> Result<SalaDto, String> {
    service
        .activar_sala(&id)
        .await
        .map(|sala| sala.into())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn desactivar_sala(
    id: String,
    service: State<'_, SharedSalaService>,
) -> Result<SalaDto, String> {
    service
        .desactivar_sala(&id)
        .await
        .map(|sala| sala.into())
        .map_err(|e| e.to_string())
}
