mod backend;
pub mod commands;
mod models;

use backend::BackendApi;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let base_url =
        std::env::var("BACKEND_BASE_URL").unwrap_or_else(|_| "http://localhost:3000/api".into());
    let backend_api = BackendApi::new(base_url);

    println!(
        "[app-desktop] Backend objetivo: {}",
        backend_api.base_url()
    );

    tauri::Builder::default()  .invoke_handler(tauri::generate_handler![
            commands::crear_sala,
            commands::listar_salas,
            commands::obtener_sala,
            commands::activar_sala,
            commands::desactivar_sala,
        ])
        .manage(backend_api)
        .plugin(tauri_plugin_opener::init())

        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
