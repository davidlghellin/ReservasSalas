slint::include_modules!();

use serde::{Deserialize, Serialize};
use slint::{ModelRc, VecModel};
use std::rc::Rc;

const BACKEND_URL: &str = "http://localhost:3000/api";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SalaDto {
    id: String,
    nombre: String,
    capacidad: u32,
    activa: bool,
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    // Modelo reactivo de salas
    let salas_model: Rc<VecModel<Sala>> = Rc::new(VecModel::default());
    ui.set_salas(ModelRc::from(salas_model.clone()));

    // Cargar salas al iniciar
    cargar_salas_ui(&ui, &salas_model);

    // Callback: Crear sala
    {
        let ui_weak = ui.as_weak();
        let salas_model = salas_model.clone();

        ui.on_crear_sala(move |nombre, capacidad| {
            let ui = ui_weak.unwrap();
            ui.set_loading(true);
            ui.set_mensaje("".into());

            if nombre.is_empty() {
                ui.set_mensaje("❌ El nombre no puede estar vacío".into());
                ui.set_loading(false);
                return;
            }

            if capacidad <= 0 {
                ui.set_mensaje("❌ La capacidad debe ser mayor que 0".into());
                ui.set_loading(false);
                return;
            }

            match crear_sala(&nombre.to_string(), capacidad as u32) {
                Ok(_) => {
                    ui.set_mensaje(format!("✅ Sala '{}' creada correctamente", nombre).into());
                    ui.set_nuevo_nombre("".into());
                    ui.set_nueva_capacidad(10);

                    // Recargar salas
                    cargar_salas_ui(&ui, &salas_model);
                }
                Err(e) => {
                    ui.set_mensaje(format!("❌ Error al crear sala: {}", e).into());
                }
            }

            ui.set_loading(false);
        });
    }

    // Callback: Activar sala
    {
        let ui_weak = ui.as_weak();
        let salas_model = salas_model.clone();

        ui.on_activar_sala(move |id| {
            let ui = ui_weak.unwrap();
            ui.set_loading(true);

            match activar_sala(&id.to_string()) {
                Ok(_) => {
                    ui.set_mensaje("✅ Sala activada correctamente".into());
                    cargar_salas_ui(&ui, &salas_model);
                }
                Err(e) => {
                    ui.set_mensaje(format!("❌ Error al activar sala: {}", e).into());
                }
            }

            ui.set_loading(false);
        });
    }

    // Callback: Desactivar sala
    {
        let ui_weak = ui.as_weak();
        let salas_model = salas_model.clone();

        ui.on_desactivar_sala(move |id| {
            let ui = ui_weak.unwrap();
            ui.set_loading(true);

            match desactivar_sala(&id.to_string()) {
                Ok(_) => {
                    ui.set_mensaje("✅ Sala desactivada correctamente".into());
                    cargar_salas_ui(&ui, &salas_model);
                }
                Err(e) => {
                    ui.set_mensaje(format!("❌ Error al desactivar sala: {}", e).into());
                }
            }

            ui.set_loading(false);
        });
    }

    // Callback: Cargar salas
    {
        let ui_weak = ui.as_weak();
        let salas_model = salas_model.clone();

        ui.on_cargar_salas(move || {
            let ui = ui_weak.unwrap();
            cargar_salas_ui(&ui, &salas_model);
        });
    }

    ui.run()
}

fn cargar_salas_ui(ui: &AppWindow, model: &Rc<VecModel<Sala>>) {
    match listar_salas() {
        Ok(salas) => {
            model.set_vec(
                salas
                    .into_iter()
                    .map(|s| Sala {
                        id: s.id.into(),
                        nombre: s.nombre.into(),
                        capacidad: s.capacidad as i32,
                        activa: s.activa,
                    })
                    .collect::<Vec<_>>(),
            );
        }
        Err(e) => {
            ui.set_mensaje(format!("❌ Error al cargar salas: {}", e).into());
        }
    }
}

// API functions usando reqwest bloqueante
fn listar_salas() -> Result<Vec<SalaDto>, String> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(format!("{}/salas", BACKEND_URL))
        .send()
        .map_err(|e| format!("Error de conexión: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Error HTTP: {}", response.status()));
    }

    response
        .json::<Vec<SalaDto>>()
        .map_err(|e| format!("Error al parsear respuesta: {}", e))
}

fn crear_sala(nombre: &str, capacidad: u32) -> Result<SalaDto, String> {
    let client = reqwest::blocking::Client::new();
    let body = serde_json::json!({
        "nombre": nombre,
        "capacidad": capacidad
    });

    let response = client
        .post(format!("{}/salas", BACKEND_URL))
        .json(&body)
        .send()
        .map_err(|e| format!("Error de conexión: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Error HTTP: {}", response.status()));
    }

    response
        .json::<SalaDto>()
        .map_err(|e| format!("Error al parsear respuesta: {}", e))
}

fn activar_sala(id: &str) -> Result<SalaDto, String> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .put(format!("{}/salas/{}/activar", BACKEND_URL, id))
        .send()
        .map_err(|e| format!("Error de conexión: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Error HTTP: {}", response.status()));
    }

    response
        .json::<SalaDto>()
        .map_err(|e| format!("Error al parsear respuesta: {}", e))
}

fn desactivar_sala(id: &str) -> Result<SalaDto, String> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .put(format!("{}/salas/{}/desactivar", BACKEND_URL, id))
        .send()
        .map_err(|e| format!("Error de conexión: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Error HTTP: {}", response.status()));
    }

    response
        .json::<SalaDto>()
        .map_err(|e| format!("Error al parsear respuesta: {}", e))
}
