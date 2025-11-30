mod backend;
pub mod commands;
mod logger;
mod models;

use backend::BackendApi;
use logger::Logger;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Inicializar logger
    let logger = Logger::new().expect("Failed to initialize logger");

    logger.info("=== Iniciando aplicación Tauri ===");
    logger.info(&format!("Logs guardados en: {}", logger.log_path().display()));

    let base_url =
        std::env::var("BACKEND_BASE_URL").unwrap_or_else(|_| "http://localhost:3000/api".into());
    let backend_api = BackendApi::new(base_url);

    logger.info(&format!("Backend objetivo: {}", backend_api.base_url()));

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::crear_sala,
            commands::listar_salas,
            commands::obtener_sala,
            commands::activar_sala,
            commands::desactivar_sala,
            commands::get_log_path,
            commands::login_usuario,
            commands::logout_usuario,
        ])
        .manage(backend_api)
        .manage(logger)
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
