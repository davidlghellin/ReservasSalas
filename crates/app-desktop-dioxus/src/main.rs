#![allow(non_snake_case)]

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

const BACKEND_URL: &str = "http://localhost:3000/api";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct SalaDto {
    id: String,
    nombre: String,
    capacidad: u32,
    activa: bool,
}

fn main() {
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).expect("failed to init logger");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut salas = use_signal(|| Vec::<SalaDto>::new());
    let mut nuevo_nombre = use_signal(|| String::new());
    let mut nueva_capacidad = use_signal(|| String::from("10"));
    let mut mensaje = use_signal(|| String::new());
    let mut loading = use_signal(|| false);

    // Cargar salas al iniciar
    use_effect(move || {
        spawn(async move {
            if let Ok(salas_data) = listar_salas().await {
                salas.set(salas_data);
            }
        });
    });

    // Handler para crear sala
    let crear_sala_handler = move |_| {
        spawn(async move {
            loading.set(true);
            mensaje.set(String::new());

            let nombre = nuevo_nombre.read().clone();
            let capacidad_str = nueva_capacidad.read().clone();

            if nombre.is_empty() {
                mensaje.set("❌ El nombre no puede estar vacío".to_string());
                loading.set(false);
                return;
            }

            let capacidad = match capacidad_str.parse::<u32>() {
                Ok(c) if c > 0 => c,
                _ => {
                    mensaje.set("❌ La capacidad debe ser un número mayor que 0".to_string());
                    loading.set(false);
                    return;
                }
            };

            match crear_sala(&nombre, capacidad).await {
                Ok(_) => {
                    mensaje.set(format!("✅ Sala '{}' creada correctamente", nombre));
                    nuevo_nombre.set(String::new());
                    nueva_capacidad.set(String::from("10"));

                    // Recargar salas
                    if let Ok(salas_data) = listar_salas().await {
                        salas.set(salas_data);
                    }
                }
                Err(e) => {
                    mensaje.set(format!("❌ Error al crear sala: {}", e));
                }
            }

            loading.set(false);
        });
    };

    // Handler para activar sala
    let activar_handler = move |id: String| {
        spawn(async move {
            loading.set(true);
            match activar_sala(&id).await {
                Ok(_) => {
                    mensaje.set("✅ Sala activada correctamente".to_string());
                    if let Ok(salas_data) = listar_salas().await {
                        salas.set(salas_data);
                    }
                }
                Err(e) => {
                    mensaje.set(format!("❌ Error al activar sala: {}", e));
                }
            }
            loading.set(false);
        });
    };

    // Handler para desactivar sala
    let desactivar_handler = move |id: String| {
        spawn(async move {
            loading.set(true);
            match desactivar_sala(&id).await {
                Ok(_) => {
                    mensaje.set("✅ Sala desactivada correctamente".to_string());
                    if let Ok(salas_data) = listar_salas().await {
                        salas.set(salas_data);
                    }
                }
                Err(e) => {
                    mensaje.set(format!("❌ Error al desactivar sala: {}", e));
                }
            }
            loading.set(false);
        });
    };

    rsx! {
        style { {include_str!("../assets/style.css")} }

        div { class: "container",
            h1 { class: "title", "🏢 Gestión de Salas" }
            p { class: "subtitle", "Sistema de reservas - Dioxus Desktop" }

            // Banner informativo
            div { class: "banner",
                "📋 Backend: {BACKEND_URL}"
            }

            // Mensaje de feedback
            if !mensaje.read().is_empty() {
                div { class: "mensaje",
                    "{mensaje}"
                }
            }

            // Formulario crear sala
            div { class: "form-container",
                h2 { "➕ Nueva Sala" }

                form { class: "form",
                    onsubmit: move |e| {
                        e.prevent_default();
                        crear_sala_handler(());
                    },

                    div { class: "form-group",
                        label { r#for: "nombre", "Nombre:" }
                        input {
                            id: "nombre",
                            r#type: "text",
                            placeholder: "Ej: Sala de conferencias",
                            value: "{nuevo_nombre}",
                            oninput: move |e| nuevo_nombre.set(e.value()),
                            disabled: *loading.read(),
                        }
                    }

                    div { class: "form-group",
                        label { r#for: "capacidad", "Capacidad:" }
                        input {
                            id: "capacidad",
                            r#type: "number",
                            min: "1",
                            value: "{nueva_capacidad}",
                            oninput: move |e| nueva_capacidad.set(e.value()),
                            disabled: *loading.read(),
                        }
                    }

                    button {
                        r#type: "submit",
                        class: "btn btn-primary",
                        disabled: *loading.read(),
                        if *loading.read() {
                            "⏳ Creando..."
                        } else {
                            "➕ Crear Sala"
                        }
                    }
                }
            }

            // Lista de salas
            div { class: "salas-container",
                h2 { "📋 Lista de Salas ({salas.read().len()})" }

                if salas.read().is_empty() {
                    div { class: "empty-state",
                        "No hay salas registradas. Crea una nueva sala para comenzar."
                    }
                } else {
                    div { class: "salas-grid",
                        for sala in salas.read().iter() {
                            div {
                                key: "{sala.id}",
                                class: if sala.activa { "sala-card activa" } else { "sala-card" },

                                div { class: "sala-header",
                                    h3 { "{sala.nombre}" }
                                    span {
                                        class: if sala.activa { "badge badge-activa" } else { "badge badge-inactiva" },
                                        if sala.activa { "✅ Activa" } else { "⏸️ Inactiva" }
                                    }
                                }

                                div { class: "sala-body",
                                    p { "👥 Capacidad: {sala.capacidad} personas" }
                                    p { class: "sala-id", "ID: {sala.id}" }
                                }

                                div { class: "sala-actions",
                                    if sala.activa {
                                        button {
                                            class: "btn btn-secondary",
                                            disabled: *loading.read(),
                                            onclick: {
                                                let id = sala.id.clone();
                                                move |_| desactivar_handler(id.clone())
                                            },
                                            "⏸️ Desactivar"
                                        }
                                    } else {
                                        button {
                                            class: "btn btn-success",
                                            disabled: *loading.read(),
                                            onclick: {
                                                let id = sala.id.clone();
                                                move |_| activar_handler(id.clone())
                                            },
                                            "▶️ Activar"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// API functions
async fn listar_salas() -> Result<Vec<SalaDto>, String> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/salas", BACKEND_URL))
        .send()
        .await
        .map_err(|e| format!("Error de conexión: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Error HTTP: {}", response.status()));
    }

    response
        .json::<Vec<SalaDto>>()
        .await
        .map_err(|e| format!("Error al parsear respuesta: {}", e))
}

async fn crear_sala(nombre: &str, capacidad: u32) -> Result<SalaDto, String> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "nombre": nombre,
        "capacidad": capacidad
    });

    let response = client
        .post(format!("{}/salas", BACKEND_URL))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Error de conexión: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Error HTTP: {}", response.status()));
    }

    response
        .json::<SalaDto>()
        .await
        .map_err(|e| format!("Error al parsear respuesta: {}", e))
}

async fn activar_sala(id: &str) -> Result<SalaDto, String> {
    let client = reqwest::Client::new();
    let response = client
        .put(format!("{}/salas/{}/activar", BACKEND_URL, id))
        .send()
        .await
        .map_err(|e| format!("Error de conexión: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Error HTTP: {}", response.status()));
    }

    response
        .json::<SalaDto>()
        .await
        .map_err(|e| format!("Error al parsear respuesta: {}", e))
}

async fn desactivar_sala(id: &str) -> Result<SalaDto, String> {
    let client = reqwest::Client::new();
    let response = client
        .put(format!("{}/salas/{}/desactivar", BACKEND_URL, id))
        .send()
        .await
        .map_err(|e| format!("Error de conexión: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Error HTTP: {}", response.status()));
    }

    response
        .json::<SalaDto>()
        .await
        .map_err(|e| format!("Error al parsear respuesta: {}", e))
}
