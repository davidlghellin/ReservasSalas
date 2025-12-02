use chrono::Local;
use dioxus::prelude::*;
use reservas_grpc::proto::Reserva as ProtoReserva;

use crate::calendario::{CalendarioDiario, CalendarioSemanal, VistaCalendario};
use crate::models::{AppState, SalaDto, Tab, UsuarioInfo, BACKEND_URL};
use crate::services::{
    activar_sala, cancelar_reserva, crear_reserva, crear_sala, desactivar_sala, listar_reservas,
    listar_salas,
};

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

    // Estado de pestañas
    let mut tab_actual = use_signal(|| Tab::Salas);

    // Estado para crear reservas
    let mut sala_seleccionada = use_signal(String::new);
    let mut fecha_inicio = use_signal(String::new);
    let mut fecha_fin = use_signal(String::new);

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
    let recargar_salas_handler = move |_| {
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

    // Handler para crear reserva
    let crear_reserva_handler = move |_| {
        let token_sig = token;
        let usuario_id = usuario.id.clone();
        spawn(async move {
            loading.set(true);
            mensaje.set(String::new());

            let sala_id = sala_seleccionada.read().clone();
            let inicio = fecha_inicio.read().clone();
            let fin = fecha_fin.read().clone();

            if sala_id.is_empty() {
                mensaje.set("❌ Debes seleccionar una sala".to_string());
                loading.set(false);
                return;
            }

            if inicio.is_empty() || fin.is_empty() {
                mensaje.set("❌ Debes ingresar fecha de inicio y fin".to_string());
                loading.set(false);
                return;
            }

            // Convertir formato datetime-local (YYYY-MM-DDTHH:MM) a RFC3339 (YYYY-MM-DDTHH:MM:SSZ)
            let inicio_iso = if inicio.matches(':').count() == 1 {
                format!("{}:00Z", inicio)
            } else if !inicio.ends_with('Z') {
                format!("{}Z", inicio)
            } else {
                inicio
            };
            let fin_iso = if fin.matches(':').count() == 1 {
                format!("{}:00Z", fin)
            } else if !fin.ends_with('Z') {
                format!("{}Z", fin)
            } else {
                fin
            };

            let token_val = token_sig.read().clone();
            if let Some(tok) = token_val {
                match crear_reserva(&sala_id, &usuario_id, &inicio_iso, &fin_iso, &tok).await {
                    Ok(_) => {
                        mensaje.set("✅ Reserva creada correctamente".to_string());
                        sala_seleccionada.set(String::new());
                        fecha_inicio.set(String::new());
                        fecha_fin.set(String::new());

                        // Recargar reservas
                        if let Ok(reservas_data) = listar_reservas(&tok).await {
                            reservas.set(reservas_data);
                        }
                    }
                    Err(e) => {
                        mensaje.set(format!("❌ Error al crear reserva: {}", e));
                    }
                }
            } else {
                mensaje.set("❌ Error: No hay token de autenticación".to_string());
            }
            loading.set(false);
        });
    };

    // Handler para cancelar reserva
    let cancelar_reserva_handler = move |id: String| {
        let token_sig = token;
        spawn(async move {
            loading.set(true);
            let token_val = token_sig.read().clone();
            if let Some(tok) = token_val {
                match cancelar_reserva(&id, &tok).await {
                    Ok(_) => {
                        mensaje.set("✅ Reserva cancelada correctamente".to_string());
                        if let Ok(reservas_data) = listar_reservas(&tok).await {
                            reservas.set(reservas_data);
                        }
                    }
                    Err(e) => {
                        mensaje.set(format!("❌ Error al cancelar reserva: {}", e));
                    }
                }
            } else {
                mensaje.set("❌ Error: No hay token de autenticación".to_string());
            }
            loading.set(false);
        });
    };

    // Handler para recargar reservas
    let recargar_reservas_handler = move |_| {
        let token_sig = token;
        spawn(async move {
            loading.set(true);
            let token_val = token_sig.read().clone();
            if let Some(tok) = token_val {
                if let Ok(reservas_data) = listar_reservas(&tok).await {
                    reservas.set(reservas_data);
                    mensaje.set("✅ Reservas actualizadas".to_string());
                } else {
                    mensaje.set("❌ Error al actualizar reservas".to_string());
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
            // Header con información del usuario
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

            // Tabs de navegación
            div { class: "tabs",
                button {
                    class: if *tab_actual.read() == Tab::Salas { "tab tab-active" } else { "tab" },
                    onclick: move |_| tab_actual.set(Tab::Salas),
                    if *tab_actual.read() == Tab::Salas {
                        "🏢 Salas ✓"
                    } else {
                        "🏢 Salas"
                    }
                }
                button {
                    class: if *tab_actual.read() == Tab::Reservas { "tab tab-active" } else { "tab" },
                    onclick: move |_| tab_actual.set(Tab::Reservas),
                    if *tab_actual.read() == Tab::Reservas {
                        "📅 Reservas ✓"
                    } else {
                        "📅 Reservas"
                    }
                }
                button {
                    class: if *tab_actual.read() == Tab::Calendario { "tab tab-active" } else { "tab" },
                    onclick: move |_| tab_actual.set(Tab::Calendario),
                    if *tab_actual.read() == Tab::Calendario {
                        "📆 Calendario ✓"
                    } else {
                        "📆 Calendario"
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

            // Contenido según la pestaña actual
            match *tab_actual.read() {
                Tab::Salas => rsx! {
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
                                onclick: recargar_salas_handler,
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
                                                    class: "btn btn-primary",
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
                },
                Tab::Reservas => rsx! {
                    // Formulario crear reserva
                    div { class: "form-container",
                        h2 { "➕ Nueva Reserva" }

                        div { class: "form-reserva",
                            div { class: "form-row",
                                div { class: "form-group",
                                    label { "Sala:" }
                                    div { class: "salas-selector",
                                        for sala in salas.read().iter().filter(|s| s.activa) {
                                            button {
                                                r#type: "button",
                                                class: if *sala_seleccionada.read() == sala.id {
                                                    "btn-selector btn-selector-active"
                                                } else {
                                                    "btn-selector"
                                                },
                                                onclick: {
                                                    let id = sala.id.clone();
                                                    move |_| sala_seleccionada.set(id.clone())
                                                },
                                                if *sala_seleccionada.read() == sala.id {
                                                    "✓ {sala.nombre}"
                                                } else {
                                                    "○ {sala.nombre}"
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "form-group",
                                    label { r#for: "fecha_inicio", "📅 Fecha y Hora de Inicio:" }
                                    input {
                                        id: "fecha_inicio",
                                        r#type: "datetime-local",
                                        value: "{fecha_inicio}",
                                        oninput: move |e| fecha_inicio.set(e.value()),
                                        disabled: *loading.read(),
                                    }

                                    label { r#for: "fecha_fin", "📅 Fecha y Hora de Fin:" }
                                    input {
                                        id: "fecha_fin",
                                        r#type: "datetime-local",
                                        value: "{fecha_fin}",
                                        oninput: move |e| fecha_fin.set(e.value()),
                                        disabled: *loading.read(),
                                    }

                                    button {
                                        r#type: "button",
                                        class: "btn btn-primary",
                                        disabled: *loading.read(),
                                        onclick: crear_reserva_handler,
                                        if *loading.read() {
                                            "⏳ Creando..."
                                        } else {
                                            "➕ Crear Reserva"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Lista de reservas
                    div { class: "reservas-container",
                        div { class: "salas-header",
                            h2 { "📋 Mis Reservas ({reservas.read().len()})" }
                            button {
                                class: "btn btn-secondary",
                                disabled: *loading.read(),
                                onclick: recargar_reservas_handler,
                                "🔄 Actualizar"
                            }
                        }

                        if reservas.read().is_empty() {
                            div { class: "empty-state",
                                "No hay reservas registradas."
                            }
                        } else {
                            div { class: "salas-grid",
                                for reserva in reservas.read().iter() {
                                    div {
                                        key: "{reserva.id}",
                                        class: "sala-card",

                                        div { class: "sala-header",
                                            h3 { "Reserva {reserva.id}" }
                                            span {
                                                class: match reserva.estado {
                                                    0 => "badge badge-activa",      // ACTIVA
                                                    1 => "badge badge-inactiva",    // CANCELADA
                                                    2 => "badge",                   // COMPLETADA
                                                    _ => "badge",
                                                },
                                                match reserva.estado {
                                                    0 => "✅ ACTIVA",
                                                    1 => "❌ CANCELADA",
                                                    2 => "✔️ COMPLETADA",
                                                    _ => "DESCONOCIDO",
                                                }
                                            }
                                        }

                                        div { class: "sala-body",
                                            p { "🏢 Sala: {reserva.sala_id}" }
                                            p { "👤 Usuario: {reserva.usuario_id}" }
                                            p { "📅 Inicio: {reserva.fecha_inicio}" }
                                            p { "📅 Fin: {reserva.fecha_fin}" }
                                        }

                                        if reserva.estado == 0 {  // ACTIVA
                                            div { class: "sala-actions",
                                                button {
                                                    class: "btn btn-secondary",
                                                    disabled: *loading.read(),
                                                    onclick: {
                                                        let id = reserva.id.clone();
                                                        move |_| cancelar_reserva_handler(id.clone())
                                                    },
                                                    "❌ Cancelar"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                Tab::Calendario => rsx! {
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
                }
            }
        }
    }
}
