use salas_application::{SalaService, SalaServiceImpl};
use salas_infrastructure::InMemorySalaRepository;
use std::sync::Arc;

pub mod commands;

pub type SharedSalaService = Arc<dyn SalaService + Send + Sync>;

pub fn create_service() -> SharedSalaService {
    let repository = InMemorySalaRepository::new();
    Arc::new(SalaServiceImpl::new(repository))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let service = create_service();

    tauri::Builder::default()
        .manage(service)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::crear_sala,
            commands::listar_salas,
            commands::obtener_sala,
            commands::activar_sala,
            commands::desactivar_sala,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
