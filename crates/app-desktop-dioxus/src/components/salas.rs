use chrono::Local;
use dioxus::prelude::*;
use reservas_grpc::proto::Reserva as ProtoReserva;

use crate::calendario::{CalendarioDiario, CalendarioSemanal, VistaCalendario};
use crate::models::{AppState, SalaDto, UsuarioInfo, BACKEND_URL};
use crate::services::{activar_sala, crear_sala, desactivar_sala, listar_reservas, listar_salas};

#[component]
pub fn SalasApp(
    usuario: UsuarioInfo,
    token: Signal<Option<String>>,
    mut app_state: Signal<AppState>,
    mut usuario_actual: Signal<Option<UsuarioInfo>>,
) -> Element {
    let mut salas = use_signal(Vec::<SalaDto>::new);
    let mut reservas = use_signal(Vec::<ProtoReserva>::new);
    let mut nuevo_nombre = use_signal(String::new);
    let mut nueva_capacidad = use_signal(|| String::from("10"));
    let mut mensaje = use_signal(String::new);
    let mut loading = use_signal(|| false);
    let mut vista_actual = use_signal(|| VistaCalendario::Diaria);
    let fecha_seleccionada = use_signal(|| Local::now());

    // Cargar salas y reservas al iniciar
    use_effect(move || {
        let token_val = token.read().clone();
        if let Some(tok) = token_val {
            spawn(async move {
                if let Ok(salas_data) = listar_salas(&tok).await {
                    salas.set(salas_data);
                }
                if let Ok(reservas_data) = listar_reservas(&tok).await {
                    reservas.set(reservas_data);
                }
            });
        }
    });

    // Handler para crear sala
    let crear_sala_handler = move |_| {
        let _token_sig = token;
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

            let token_val = token.read().clone();
            if let Some(tok) = token_val {
                match crear_sala(&nombre, capacidad, &tok).await {
                    Ok(_) => {
                        mensaje.set(format!("✅ Sala '{}' creada correctamente", nombre));
                        nuevo_nombre.set(String::new());
                        nueva_capacidad.set(String::from("10"));

                        // Recargar salas
                        if let Ok(salas_data) = listar_salas(&tok).await {
                            salas.set(salas_data);
                        }
                    }
                    Err(e) => {
                        mensaje.set(format!("❌ Error al crear sala: {}", e));
                    }
                }
            } else {
                mensaje.set("❌ Error: No hay token de autenticación".to_string());
            }
            loading.set(false);
        });
    };

    // Handler para activar sala
    let activar_handler = move |id: String| {
        let token_sig = token;
        spawn(async move {
            loading.set(true);
            let token_val = token_sig.read().clone();
            if let Some(tok) = token_val {
                match activar_sala(&id, &tok).await {
                    Ok(_) => {
                        mensaje.set("✅ Sala activada correctamente".to_string());
                        if let Ok(salas_data) = listar_salas(&tok).await {
                            salas.set(salas_data);
                        }
                    }
                    Err(e) => {
                        mensaje.set(format!("❌ Error al activar sala: {}", e));
                    }
                }
            } else {
                mensaje.set("❌ Error: No hay token de autenticación".to_string());
            }
            loading.set(false);
        });
    };

    // Handler para desactivar sala
    let desactivar_handler = move |id: String| {
        let token_sig = token;
        spawn(async move {
            loading.set(true);
            let token_val = token_sig.read().clone();
            if let Some(tok) = token_val {
                match desactivar_sala(&id, &tok).await {
                    Ok(_) => {
                        mensaje.set("✅ Sala desactivada correctamente".to_string());
                        if let Ok(salas_data) = listar_salas(&tok).await {
                            salas.set(salas_data);
                        }
                    }
                    Err(e) => {
                        mensaje.set(format!("❌ Error al desactivar sala: {}", e));
                    }
                }
            } else {
                mensaje.set("❌ Error: No hay token de autenticación".to_string());
            }
            loading.set(false);
        });
    };

    // Handler para recargar salas
    let recargar_handler = move |_| {
        let token_sig = token;
        spawn(async move {
            loading.set(true);
            let token_val = token_sig.read().clone();
            if let Some(tok) = token_val {
                if let Ok(salas_data) = listar_salas(&tok).await {
                    salas.set(salas_data);
                    mensaje.set("✅ Salas actualizadas".to_string());
                } else {
                    mensaje.set("❌ Error al actualizar salas".to_string());
                }
            } else {
                mensaje.set("❌ Error: No hay token de autenticación".to_string());
            }
            loading.set(false);
        });
    };

    rsx! {
        style { {include_str!("../../assets/style.css")} }

        div { class: "container",
            div { class: "header-with-user",
                div {
                    h1 { class: "title", "🏢 Gestión de Salas" }
                    p { class: "subtitle", "Sistema de reservas - Dioxus Desktop" }
                }
                div { class: "user-info",
                    div { class: "user-name", "👤 {usuario.nombre}" }
                    div { class: "user-email", "📧 {usuario.email}" }
                    div { class: "user-rol", "🎫 {usuario.rol}" }
                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| {
                            token.set(None);
                            usuario_actual.set(None);
                            app_state.set(AppState::Login);
                        },
                        "🚪 Salir"
                    }
                }
            }

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

            // Selector de vista
            div { class: "vista-selector",
                button {
                    class: if *vista_actual.read() == VistaCalendario::Diaria {
                        "btn btn-primary"
                    } else {
                        "btn btn-secondary"
                    },
                    onclick: move |_| vista_actual.set(VistaCalendario::Diaria),
                    "📅 Vista Diaria"
                }
                button {
                    class: if *vista_actual.read() == VistaCalendario::Semanal {
                        "btn btn-primary"
                    } else {
                        "btn btn-secondary"
                    },
                    onclick: move |_| vista_actual.set(VistaCalendario::Semanal),
                    "📆 Vista Semanal"
                }
                button {
                    class: "btn btn-secondary",
                    onclick: move |_| {
                        let token_val = token.read().clone();
                        spawn(async move {
                            if let Some(tok) = token_val {
                                if let Ok(reservas_data) = listar_reservas(&tok).await {
                                    reservas.set(reservas_data);
                                    mensaje.set("✅ Reservas actualizadas".to_string());
                                }
                            }
                        });
                    },
                    "🔄 Actualizar"
                }
            }

            // Calendario
            div { class: "calendario-container",
                match *vista_actual.read() {
                    VistaCalendario::Diaria => rsx! {
                        CalendarioDiario {
                            reservas: reservas.read().clone(),
                            salas: salas.read().iter().map(|s| salas_grpc::proto::SalaResponse {
                                id: s.id.clone(),
                                nombre: s.nombre.clone(),
                                capacidad: s.capacidad,
                                activa: s.activa,
                            }).collect(),
                            fecha: *fecha_seleccionada.read(),
                        }
                    },
                    VistaCalendario::Semanal => rsx! {
                        CalendarioSemanal {
                            reservas: reservas.read().clone(),
                            salas: salas.read().iter().map(|s| salas_grpc::proto::SalaResponse {
                                id: s.id.clone(),
                                nombre: s.nombre.clone(),
                                capacidad: s.capacidad,
                                activa: s.activa,
                            }).collect(),
                            fecha_inicio: *fecha_seleccionada.read(),
                        }
                    },
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
                div { class: "salas-header",
                    h2 { "📋 Lista de Salas ({salas.read().len()})" }
                    button {
                        class: "btn btn-secondary",
                        disabled: *loading.read(),
                        onclick: recargar_handler,
                        "🔄 Actualizar"
                    }
                }

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
