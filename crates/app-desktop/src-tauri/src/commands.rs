use crate::backend::BackendApi;
use crate::models::{CrearSalaRequest, SalaDto};
use tauri::State;

#[tauri::command]
pub async fn crear_sala(
    request: CrearSalaRequest,
    backend: State<'_, BackendApi>,
) -> Result<SalaDto, String> {
    backend.crear_sala(request).await
}

#[tauri::command]
pub async fn listar_salas(backend: State<'_, BackendApi>) -> Result<Vec<SalaDto>, String> {
    println!("[app-desktop] Listar salas");
    match backend.listar_salas().await {
        Ok(salas) => {
            println!("[listar_salas] obtenido {} salas", salas.len());
            Ok(salas)
        }
        Err(e) => {
            eprintln!("[listar_salas] error: {:?}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn obtener_sala(
    id: String,
    backend: State<'_, BackendApi>,
) -> Result<Option<SalaDto>, String> {
    backend.obtener_sala(&id).await
}

#[tauri::command]
pub async fn activar_sala(
    id: String,
    backend: State<'_, BackendApi>,
) -> Result<SalaDto, String> {
    backend.activar_sala(&id).await
}

#[tauri::command]
pub async fn desactivar_sala(
    id: String,
    backend: State<'_, BackendApi>,
) -> Result<SalaDto, String> {
    backend.desactivar_sala(&id).await
}
