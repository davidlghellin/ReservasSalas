#![allow(non_snake_case)]

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use usuarios_grpc::proto::{usuario_service_client::UsuarioServiceClient, LoginRequest};
use salas_grpc::proto::{
    sala_service_client::SalaServiceClient, ActivarSalaRequest, CrearSalaRequest,
    DesactivarSalaRequest, ListarSalasRequest,
};
use tonic::{metadata::MetadataValue, Request};

const BACKEND_URL: &str = "http://localhost:3000/api";
const GRPC_URL: &str = "http://localhost:50051";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct SalaDto {
    id: String,
    nombre: String,
    capacidad: u32,
    activa: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct UsuarioInfo {
    id: String,
    nombre: String,
    email: String,
    rol: String,
}

#[derive(Debug, Clone, PartialEq)]
enum AppState {
    Login,
    Authenticated(UsuarioInfo),
}

fn main() {
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).expect("failed to init logger");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut app_state = use_signal(|| AppState::Login);
    let mut token = use_signal(|| Option::<String>::None);
    let mut usuario_actual = use_signal(|| Option::<UsuarioInfo>::None);

    let current_state = app_state.read().clone();
    
    match current_state {
        AppState::Login => {
            rsx! {
                LoginScreen {
                    app_state: app_state,
                    token: token,
                    usuario_actual: usuario_actual,
                }
            }
        }
        AppState::Authenticated(_) => {
            let usuario = usuario_actual.read().clone();
            if let Some(usuario) = usuario {
                rsx! {
                    SalasApp {
                        usuario: usuario.clone(),
                        token: token,
                        app_state: app_state,
                        usuario_actual: usuario_actual,
                    }
                }
            } else {
                rsx! { div { "Error: Usuario no encontrado" } }
            }
        }
    }
}

#[component]
fn LoginScreen(
    mut app_state: Signal<AppState>,
    mut token: Signal<Option<String>>,
    mut usuario_actual: Signal<Option<UsuarioInfo>>,
) -> Element {
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut error = use_signal(|| String::new());
    let mut loading = use_signal(|| false);

    let login_handler = move |_| {
        spawn(async move {
            let email_val = email.read().clone();
            let password_val = password.read().clone();

            if email_val.is_empty() || password_val.is_empty() {
                error.set("Email y contraseña son requeridos".to_string());
                return;
            }

            loading.set(true);
            error.set(String::new());

            match login_usuario(&email_val, &password_val).await {
                Ok((usuario, tok)) => {
                    loading.set(false);
                    token.set(Some(tok));
                    usuario_actual.set(Some(usuario.clone()));
                    app_state.set(AppState::Authenticated(usuario));
                }
                Err(e) => {
                    loading.set(false);
                    error.set(e);
                }
            }
        });
    };

    rsx! {
        style { {include_str!("../assets/style.css")} }

        div { class: "login-container",
            div { class: "login-box",
                h1 { class: "login-title", "🔐 Iniciar Sesión" }
                p { class: "login-subtitle", "Sistema de Gestión de Salas" }

                if !error.read().is_empty() {
                    div { class: "error-message",
                        "{error}"
                    }
                }

                form { class: "login-form",
                    onsubmit: move |e| {
                        e.prevent_default();
                        login_handler(());
                    },

                    div { class: "form-group",
                        label { r#for: "email", "Email:" }
                        input {
                            id: "email",
                            r#type: "email",
                            placeholder: "usuario@ejemplo.com",
                            value: "{email}",
                            oninput: move |e| email.set(e.value()),
                            disabled: *loading.read(),
                        }
                    }

                    div { class: "form-group",
                        label { r#for: "password", "Contraseña:" }
                        input {
                            id: "password",
                            r#type: "password",
                            placeholder: "••••••••",
                            value: "{password}",
                            oninput: move |e| password.set(e.value()),
                            disabled: *loading.read(),
                        }
                    }

                    button {
                        r#type: "submit",
                        class: "btn btn-primary btn-block",
                        disabled: *loading.read(),
                        if *loading.read() {
                            "⏳ Iniciando sesión..."
                        } else {
                            "🚀 Iniciar Sesión"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SalasApp(
    usuario: UsuarioInfo,
    token: Signal<Option<String>>,
    mut app_state: Signal<AppState>,
    mut usuario_actual: Signal<Option<UsuarioInfo>>,
) -> Element {
    let mut salas = use_signal(|| Vec::<SalaDto>::new());
    let mut nuevo_nombre = use_signal(|| String::new());
    let mut nueva_capacidad = use_signal(|| String::from("10"));
    let mut mensaje = use_signal(|| String::new());
    let mut loading = use_signal(|| false);

    // Cargar salas al iniciar
    use_effect(move || {
        let token_val = token.read().clone();
        if let Some(tok) = token_val {
            spawn(async move {
                if let Ok(salas_data) = listar_salas(&tok).await {
                    salas.set(salas_data);
                }
            });
        }
    });

    // Handler para crear sala
    let crear_sala_handler = move |_| {
        let token_sig = token;
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
        style { {include_str!("../assets/style.css")} }

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

// API functions - Login
async fn login_usuario(email: &str, password: &str) -> Result<(UsuarioInfo, String), String> {
    let mut client = UsuarioServiceClient::connect(GRPC_URL)
        .await
        .map_err(|e| format!("Error de conexión gRPC: {}", e))?;

    let request = Request::new(LoginRequest {
        email: email.to_string(),
        password: password.to_string(),
    });

    let response = client.login(request).await
        .map_err(|e| format!("Error al hacer login: {}", e))?;

    let login_response = response.into_inner();
    let usuario_proto = login_response.usuario.ok_or_else(|| "Respuesta sin usuario".to_string())?;

    let usuario = UsuarioInfo {
        id: usuario_proto.id,
        nombre: usuario_proto.nombre,
        email: usuario_proto.email,
        rol: usuario_proto.rol,
    };

    Ok((usuario, login_response.token))
}

// API functions - Salas (gRPC)
async fn listar_salas(token: &str) -> Result<Vec<SalaDto>, String> {
    let mut client = SalaServiceClient::connect(GRPC_URL)
        .await
        .map_err(|e| format!("Error de conexión gRPC: {}", e))?;

    let mut request = Request::new(ListarSalasRequest {});

    // Agregar token JWT
    let auth_value = MetadataValue::try_from(format!("Bearer {}", token))
        .map_err(|e| format!("Error al crear header: {}", e))?;
    request.metadata_mut().insert("authorization", auth_value);

    let response = client
        .listar_salas(request)
        .await
        .map_err(|e| format!("Error gRPC: {}", e))?;

    let salas = response
        .into_inner()
        .salas
        .into_iter()
        .map(|s| SalaDto {
            id: s.id,
            nombre: s.nombre,
            capacidad: s.capacidad,
            activa: s.activa,
        })
        .collect();

    Ok(salas)
}

async fn crear_sala(nombre: &str, capacidad: u32, token: &str) -> Result<SalaDto, String> {
    let mut client = SalaServiceClient::connect(GRPC_URL)
        .await
        .map_err(|e| format!("Error de conexión gRPC: {}", e))?;

    let mut request = Request::new(CrearSalaRequest {
        nombre: nombre.to_string(),
        capacidad,
    });

    // Agregar token JWT
    let auth_value = MetadataValue::try_from(format!("Bearer {}", token))
        .map_err(|e| format!("Error al crear header: {}", e))?;
    request.metadata_mut().insert("authorization", auth_value);

    let response = client
        .crear_sala(request)
        .await
        .map_err(|e| format!("Error gRPC: {}", e))?;

    let sala = response.into_inner();

    Ok(SalaDto {
        id: sala.id,
        nombre: sala.nombre,
        capacidad: sala.capacidad,
        activa: sala.activa,
    })
}

async fn activar_sala(id: &str, token: &str) -> Result<SalaDto, String> {
    let mut client = SalaServiceClient::connect(GRPC_URL)
        .await
        .map_err(|e| format!("Error de conexión gRPC: {}", e))?;

    let mut request = Request::new(ActivarSalaRequest {
        id: id.to_string(),
    });

    // Agregar token JWT
    let auth_value = MetadataValue::try_from(format!("Bearer {}", token))
        .map_err(|e| format!("Error al crear header: {}", e))?;
    request.metadata_mut().insert("authorization", auth_value);

    let response = client
        .activar_sala(request)
        .await
        .map_err(|e| format!("Error gRPC: {}", e))?;

    let sala = response.into_inner();

    Ok(SalaDto {
        id: sala.id,
        nombre: sala.nombre,
        capacidad: sala.capacidad,
        activa: sala.activa,
    })
}

async fn desactivar_sala(id: &str, token: &str) -> Result<SalaDto, String> {
    let mut client = SalaServiceClient::connect(GRPC_URL)
        .await
        .map_err(|e| format!("Error de conexión gRPC: {}", e))?;

    let mut request = Request::new(DesactivarSalaRequest {
        id: id.to_string(),
    });

    // Agregar token JWT
    let auth_value = MetadataValue::try_from(format!("Bearer {}", token))
        .map_err(|e| format!("Error al crear header: {}", e))?;
    request.metadata_mut().insert("authorization", auth_value);

    let response = client
        .desactivar_sala(request)
        .await
        .map_err(|e| format!("Error gRPC: {}", e))?;

    let sala = response.into_inner();

    Ok(SalaDto {
        id: sala.id,
        nombre: sala.nombre,
        capacidad: sala.capacidad,
        activa: sala.activa,
    })
}
