#![allow(clippy::needless_return)]

use iced::widget::{button, column, container, row, scrollable, text, text_input, Column};
use iced::{Alignment, Element, Length, Task, Theme};

use tonic::transport::Channel;
use tonic::Request;

use salas_grpc::proto::sala_service_client::SalaServiceClient;
use salas_grpc::proto::{
    ActivarSalaRequest, CrearSalaRequest, DesactivarSalaRequest, ListarSalasRequest, SalaResponse,
};

use usuarios_grpc::proto::usuario_service_client::UsuarioServiceClient;
use usuarios_grpc::proto::{LoginRequest, LoginResponse};

use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(not(target_os = "macos"))]
use notify_rust::Notification;

const GRPC_URL: &str = "http://localhost:50051";

// Alias para simplificar el código de la UI
type SalaDto = SalaResponse;

// Tipo para el cliente compartido de salas
type SharedSalaClient = Arc<Mutex<Option<SalaServiceClient<Channel>>>>;

// Tipo para el cliente compartido de usuarios
type SharedUsuarioClient = Arc<Mutex<Option<UsuarioServiceClient<Channel>>>>;

// Cliente gRPC compartido de salas (una única conexión)
static GRPC_SALA_CLIENT: Lazy<SharedSalaClient> = Lazy::new(|| Arc::new(Mutex::new(None)));

// Cliente gRPC compartido de usuarios (una única conexión)
static GRPC_USUARIO_CLIENT: Lazy<SharedUsuarioClient> = Lazy::new(|| Arc::new(Mutex::new(None)));

// Token JWT almacenado
static JWT_TOKEN: Lazy<Arc<Mutex<Option<String>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

fn main() -> iced::Result {
    iced::application("Gestión de Salas - Iced (gRPC)", App::update, App::view)
        .theme(App::theme)
        .run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {
    // Mensajes de login
    EmailChanged(String),
    PasswordChanged(String),
    Login,
    LoginExitoso(String, UsuarioInfo),
    LoginError(String),
    Logout,
    
    // Mensajes de salas
    SalasCargadas(Result<Vec<SalaDto>, String>),
    SalaCreada(Result<SalaDto, String>),
    SalaActivada(Result<SalaDto, String>),
    SalaDesactivada(Result<SalaDto, String>),
    NombreChanged(String),
    CapacidadChanged(String),
    CrearSala,
    ActivarSala(String),
    DesactivarSala(String),
    ActualizarSalas,
}

#[derive(Debug, Clone)]
struct UsuarioInfo {
    id: String,
    nombre: String,
    email: String,
    rol: String,
}

enum AppState {
    Login {
        email: String,
        password: String,
        error: String,
        loading: bool,
    },
    Authenticated {
        usuario: UsuarioInfo,
        salas: Vec<SalaDto>,
        nuevo_nombre: String,
        nueva_capacidad: String,
        mensaje: String,
        loading: bool,
    },
}

struct App {
    state: AppState,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                state: AppState::Login {
                    email: String::new(),
                    password: String::new(),
                    error: String::new(),
                    loading: false,
                },
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Mensajes de login
            Message::EmailChanged(email) => {
                if let AppState::Login { email: e, .. } = &mut self.state {
                    *e = email;
                }
                Task::none()
            }
            Message::PasswordChanged(password) => {
                if let AppState::Login { password: p, .. } = &mut self.state {
                    *p = password;
                }
                Task::none()
            }
            Message::Login => {
                if let AppState::Login { email, password, loading, error } = &mut self.state {
                    if email.is_empty() || password.is_empty() {
                        *error = "Email y contraseña son requeridos".to_string();
                        return Task::none();
                    }
                    *loading = true;
                    *error = String::new();
                    let email = email.clone();
                    let password = password.clone();
                    Task::perform(login_usuario(email, password), |result| {
                        match result {
                            Ok((token, usuario)) => Message::LoginExitoso(token, usuario),
                            Err(e) => Message::LoginError(e),
                        }
                    })
                } else {
                    Task::none()
                }
            }
            Message::LoginExitoso(token, usuario) => {
                // Guardar token
                let token_arc = Arc::clone(&JWT_TOKEN);
                tokio::spawn(async move {
                    let mut guard = token_arc.lock().await;
                    *guard = Some(token);
                });

                // Cambiar a estado autenticado
                self.state = AppState::Authenticated {
                    usuario,
                    salas: Vec::new(),
                    nuevo_nombre: String::new(),
                    nueva_capacidad: String::from("10"),
                    mensaje: String::new(),
                    loading: false,
                };
                mostrar_notificacion(
                    "✅ Login exitoso",
                    "Bienvenido al sistema",
                    TipoNotificacion::Exito,
                );
                Task::perform(listar_salas(), Message::SalasCargadas)
            }
            Message::LoginError(error) => {
                let error_clone = error.clone();
                if let AppState::Login { error: e, loading, .. } = &mut self.state {
                    *e = error;
                    *loading = false;
                }
                mostrar_notificacion("❌ Error de login", &error_clone, TipoNotificacion::Error);
                Task::none()
            }
            Message::Logout => {
                // Limpiar token
                let token_arc = Arc::clone(&JWT_TOKEN);
                tokio::spawn(async move {
                    let mut guard = token_arc.lock().await;
                    *guard = None;
                });

                self.state = AppState::Login {
                    email: String::new(),
                    password: String::new(),
                    error: String::new(),
                    loading: false,
                };
                Task::none()
            }

            // Mensajes de salas (solo si está autenticado)
            Message::SalasCargadas(Ok(salas)) => {
                if let AppState::Authenticated { salas: s, loading, .. } = &mut self.state {
                    *s = salas;
                    *loading = false;
                }
                Task::none()
            }
            Message::SalasCargadas(Err(e)) => {
                if let AppState::Authenticated { mensaje, loading, .. } = &mut self.state {
                    *mensaje = format!("❌ Error al cargar salas: {}", e);
                    *loading = false;
                }
                Task::none()
            }
            Message::SalaCreada(Ok(_)) => {
                if let AppState::Authenticated { nuevo_nombre, nueva_capacidad, mensaje, loading, .. } = &mut self.state {
                    let nombre = nuevo_nombre.clone();
                    *mensaje = format!("✅ Sala '{}' creada correctamente", nombre);
                    mostrar_notificacion(
                        "✅ Sala creada",
                        &format!("La sala '{}' se creó correctamente", nombre),
                        TipoNotificacion::Exito,
                    );
                    nuevo_nombre.clear();
                    *nueva_capacidad = String::from("10");
                    *loading = false;
                }
                Task::perform(listar_salas(), Message::SalasCargadas)
            }
            Message::SalaCreada(Err(e)) => {
                if let AppState::Authenticated { mensaje, loading, .. } = &mut self.state {
                    *mensaje = format!("❌ Error al crear sala: {}", e);
                    *loading = false;
                }
                mostrar_notificacion("❌ Error", &e, TipoNotificacion::Error);
                Task::none()
            }
            Message::SalaActivada(Ok(_)) => {
                if let AppState::Authenticated { mensaje, loading, .. } = &mut self.state {
                    *mensaje = "✅ Sala activada correctamente".to_string();
                    *loading = false;
                }
                mostrar_notificacion(
                    "✅ Sala activada",
                    "La sala se activó correctamente",
                    TipoNotificacion::Exito,
                );
                Task::perform(listar_salas(), Message::SalasCargadas)
            }
            Message::SalaActivada(Err(e)) => {
                if let AppState::Authenticated { mensaje, loading, .. } = &mut self.state {
                    *mensaje = format!("❌ Error al activar sala: {}", e);
                    *loading = false;
                }
                mostrar_notificacion("❌ Error", &e, TipoNotificacion::Error);
                Task::none()
            }
            Message::SalaDesactivada(Ok(_)) => {
                if let AppState::Authenticated { mensaje, loading, .. } = &mut self.state {
                    *mensaje = "✅ Sala desactivada correctamente".to_string();
                    *loading = false;
                }
                mostrar_notificacion(
                    "✅ Sala desactivada",
                    "La sala se desactivó correctamente",
                    TipoNotificacion::Exito,
                );
                Task::perform(listar_salas(), Message::SalasCargadas)
            }
            Message::SalaDesactivada(Err(e)) => {
                if let AppState::Authenticated { mensaje, loading, .. } = &mut self.state {
                    *mensaje = format!("❌ Error al desactivar sala: {}", e);
                    *loading = false;
                }
                mostrar_notificacion("❌ Error", &e, TipoNotificacion::Error);
                Task::none()
            }
            Message::NombreChanged(nombre) => {
                if let AppState::Authenticated { nuevo_nombre, .. } = &mut self.state {
                    *nuevo_nombre = nombre;
                }
                Task::none()
            }
            Message::CapacidadChanged(capacidad) => {
                if let AppState::Authenticated { nueva_capacidad, .. } = &mut self.state {
                    *nueva_capacidad = capacidad;
                }
                Task::none()
            }
            Message::CrearSala => {
                if let AppState::Authenticated { nuevo_nombre, nueva_capacidad, mensaje, loading, .. } = &mut self.state {
                    if nuevo_nombre.is_empty() {
                        *mensaje = "❌ El nombre no puede estar vacío".to_string();
                        return Task::none();
                    }

                    let capacidad = match nueva_capacidad.parse::<u32>() {
                        Ok(c) if c > 0 => c,
                        _ => {
                            *mensaje = "❌ La capacidad debe ser un número mayor que 0".to_string();
                            return Task::none();
                        }
                    };

                    *loading = true;
                    mensaje.clear();
                    let nombre = nuevo_nombre.clone();
                    Task::perform(crear_sala(nombre, capacidad), Message::SalaCreada)
                } else {
                    Task::none()
                }
            }
            Message::ActivarSala(id) => {
                if let AppState::Authenticated { loading, .. } = &mut self.state {
                    *loading = true;
                }
                Task::perform(activar_sala(id), Message::SalaActivada)
            }
            Message::DesactivarSala(id) => {
                if let AppState::Authenticated { loading, .. } = &mut self.state {
                    *loading = true;
                }
                Task::perform(desactivar_sala(id), Message::SalaDesactivada)
            }
            Message::ActualizarSalas => {
                if let AppState::Authenticated { mensaje, loading, .. } = &mut self.state {
                    *loading = true;
                    mensaje.clear();
                }
                Task::perform(listar_salas(), Message::SalasCargadas)
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.state {
            AppState::Login { email, password, error, loading } => {
                self.view_login(email, password, error, *loading)
            }
            AppState::Authenticated { usuario, salas, nuevo_nombre, nueva_capacidad, mensaje, loading } => {
                self.view_main(usuario, salas, nuevo_nombre, nueva_capacidad, mensaje, *loading)
            }
        }
    }

    fn view_login<'a>(&'a self, email: &'a str, password: &'a str, error: &'a str, loading: bool) -> Element<'a, Message> {
        let title = column![
            text("🔐 Iniciar Sesión")
                .size(36)
                .width(Length::Fill)
                .center(),
            text("Sistema de Gestión de Salas")
                .size(18)
                .width(Length::Fill)
                .center(),
        ]
        .spacing(10)
        .padding(30);

        let error_message = if !error.is_empty() {
            container(
                text(error)
                    .size(14)
                    .width(Length::Fill)
                    .center()
            )
            .padding(10)
        } else {
            container(text(""))
        };

        let form = column![
            text("Email:").size(16),
            text_input("usuario@ejemplo.com", &email)
                .on_input(Message::EmailChanged)
                .padding(10)
                .width(Length::Fill),
            text("Contraseña:").size(16),
            text_input("", &password)
                .on_input(Message::PasswordChanged)
                .padding(10)
                .width(Length::Fill),
            button(text(if loading { "⏳ Iniciando sesión..." } else { "🚀 Iniciar Sesión" }))
                .on_press_maybe(if !loading { Some(Message::Login) } else { None })
                .padding(15)
                .width(Length::Fill),
        ]
        .spacing(15)
        .padding(40)
        .max_width(400);

        let content = column![
            title,
            error_message,
            form,
        ]
        .spacing(20)
        .align_x(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    fn view_main<'a>(&'a self, usuario: &'a UsuarioInfo, salas: &'a Vec<SalaDto>, nuevo_nombre: &'a str, nueva_capacidad: &'a str, mensaje: &'a str, loading: bool) -> Element<'a, Message> {
        let header = column![
            row![
                column![
                    text("🏢 Gestión de Salas")
                        .size(32)
                        .width(Length::Fill),
                    text("Sistema de reservas - Iced UI (gRPC)")
                        .size(16)
                        .width(Length::Fill),
                ]
                .width(Length::Fill),
                column![
                    text(format!("👤 {}", usuario.nombre))
                        .size(16)
                        .align_x(Alignment::End),
                    text(format!("📧 {}", usuario.email))
                        .size(14)
                        .align_x(Alignment::End),
                    text(format!("🎫 {}", usuario.rol))
                        .size(14)
                        .align_x(Alignment::End),
                    button(text("🚪 Salir"))
                        .on_press(Message::Logout)
                        .padding(8),
                ]
                .spacing(5)
                .align_x(Alignment::End),
            ]
            .spacing(20)
            .align_y(Alignment::Center),
        ]
        .spacing(5)
        .padding(20);

        let banner = container(
            text(format!("📋 Backend gRPC: {}", grpc_url()))
                .width(Length::Fill)
                .center(),
        )
        .padding(10)
        .width(Length::Fill)
        .center_x(Length::Fill);

        let mensaje_view = if !mensaje.is_empty() {
            container(text(mensaje).width(Length::Fill).center())
                .padding(15)
                .width(Length::Fill)
                .center_x(Length::Fill)
        } else {
            container(text(""))
        };

        let form = column![
            text("➕ Nueva Sala").size(20),
            row![
                text_input("Nombre de la sala", nuevo_nombre)
                    .on_input(Message::NombreChanged)
                    .padding(10)
                    .width(Length::FillPortion(3)),
                text_input("Capacidad", nueva_capacidad)
                    .on_input(Message::CapacidadChanged)
                    .padding(10)
                    .width(Length::FillPortion(1)),
                button(text(if loading {
                    "⏳ Creando..."
                } else {
                    "➕ Crear Sala"
                }))
                .on_press_maybe(if !loading {
                    Some(Message::CrearSala)
                } else {
                    None
                })
                .padding(10),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .spacing(10)
        .padding(20)
        .width(Length::Fill);

        let salas_header = row![
            text(format!("📋 Lista de Salas ({})", salas.len())).size(20),
            button(text("🔄 Actualizar"))
                .on_press_maybe(if !loading {
                    Some(Message::ActualizarSalas)
                } else {
                    None
                })
                .padding(10),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .width(Length::Fill);

        let salas_list = if salas.is_empty() {
            column![
                text("No hay salas registradas. Crea una nueva sala para comenzar.")
                    .width(Length::Fill)
                    .center()
            ]
            .padding(40)
            .width(Length::Fill)
        } else {
            Column::with_children(
                salas
                    .iter()
                    .map(|sala| {
                        let badge = if sala.activa {
                            text("✅ Activa")
                        } else {
                            text("⏸️ Inactiva")
                        };

                        let action_button = if sala.activa {
                            button(text("⏸️ Desactivar"))
                                .on_press_maybe(if !loading {
                                    Some(Message::DesactivarSala(sala.id.clone()))
                                } else {
                                    None
                                })
                                .padding(8)
                        } else {
                            button(text("▶️ Activar"))
                                .on_press_maybe(if !loading {
                                    Some(Message::ActivarSala(sala.id.clone()))
                                } else {
                                    None
                                })
                                .padding(8)
                        };

                        container(
                            row![
                                column![
                                    row![text(&sala.nombre).size(18), badge,]
                                        .spacing(10)
                                        .align_y(Alignment::Center),
                                    text(format!("👥 Capacidad: {} personas", sala.capacidad)),
                                    text(format!("ID: {}", sala.id)).size(12),
                                ]
                                .spacing(8)
                                .width(Length::Fill),
                                action_button,
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center)
                            .padding(15),
                        )
                        .padding(10)
                        .width(Length::Fill)
                        .into()
                    })
                    .collect::<Vec<_>>(),
            )
            .spacing(10)
            .width(Length::Fill)
        };

        let salas_container = column![salas_header, scrollable(salas_list)]
            .spacing(15)
            .padding(20)
            .width(Length::Fill);

        let content = column![header, banner, mensaje_view, form, salas_container]
            .spacing(20)
            .padding(20)
            .width(Length::Fill);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dracula
    }
}

// -------- Notificaciones del sistema --------

/// Muestra una notificación del sistema
fn mostrar_notificacion(titulo: &str, mensaje: &str, tipo: TipoNotificacion) {
    // En macOS, usar terminal-notifier si está disponible
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        // Sonido según el tipo de notificación
        let sound = match tipo {
            TipoNotificacion::Exito => "Glass",      // Sonido de éxito
            TipoNotificacion::Error => "Basso",      // Sonido de error
            TipoNotificacion::Info => "default",     // Sonido por defecto
        };

        // Intentar con terminal-notifier primero (más bonito en macOS)
        let result = Command::new("terminal-notifier")
            .arg("-title")
            .arg(titulo)
            .arg("-message")
            .arg(mensaje)
            .arg("-sound")
            .arg(sound)
            .spawn();

        if result.is_ok() {
            return; // terminal-notifier funcionó
        }

        // Si terminal-notifier no está instalado, usar osascript como fallback
        let _ = Command::new("osascript")
            .arg("-e")
            .arg(format!(
                "display notification \"{}\" with title \"{}\" sound name \"{}\"",
                mensaje, titulo, sound
            ))
            .spawn();
    }

    // En Linux/Windows, usar notify-rust
    #[cfg(not(target_os = "macos"))]
    {
        let _ = Notification::new()
            .summary(titulo)
            .body(mensaje)
            .icon(match tipo {
                TipoNotificacion::Exito => "dialog-information",
                TipoNotificacion::Error => "dialog-error",
                TipoNotificacion::Info => "dialog-information",
            })
            .timeout(3000)
            .show();
    }
}

#[derive(Debug, Clone, Copy)]
enum TipoNotificacion {
    Exito,
    Error,
    Info,
}

// -------- gRPC client compartido con reconexión automática --------

// -------- Clientes gRPC --------

/// Obtiene el cliente gRPC de salas, creando una nueva conexión si es necesario
async fn get_sala_client() -> Result<SalaServiceClient<Channel>, String> {
    let mut guard = GRPC_SALA_CLIENT.lock().await;

    if let Some(client) = guard.as_ref() {
        return Ok(client.clone());
    }

    // Si no hay cliente, conectar
    let client = connect_sala_grpc().await?;
    *guard = Some(client.clone());
    Ok(client)
}

/// Obtiene el cliente gRPC de usuarios, creando una nueva conexión si es necesario
async fn get_usuario_client() -> Result<UsuarioServiceClient<Channel>, String> {
    let mut guard = GRPC_USUARIO_CLIENT.lock().await;

    if let Some(client) = guard.as_ref() {
        return Ok(client.clone());
    }

    // Si no hay cliente, conectar
    let client = connect_usuario_grpc().await?;
    *guard = Some(client.clone());
    Ok(client)
}

/// Resetea el cliente gRPC de salas para forzar reconexión
async fn reset_sala_client() {
    let mut guard = GRPC_SALA_CLIENT.lock().await;
    *guard = None;
}

fn grpc_url() -> String {
    std::env::var("GRPC_URL").unwrap_or_else(|_| GRPC_URL.to_string())
}

/// Conecta al servidor gRPC de salas
async fn connect_sala_grpc() -> Result<SalaServiceClient<Channel>, String> {
    SalaServiceClient::connect(grpc_url())
        .await
        .map_err(|e| format!("Error de conexión gRPC: {}", e))
}

/// Conecta al servidor gRPC de usuarios
async fn connect_usuario_grpc() -> Result<UsuarioServiceClient<Channel>, String> {
    UsuarioServiceClient::connect(grpc_url())
        .await
        .map_err(|e| format!("Error de conexión gRPC usuarios: {}", e))
}

/// Ejecuta una operación gRPC de salas con reconexión automática en caso de fallo
async fn with_sala_retry<F, Fut, T>(operation: F) -> Result<T, String>
where
    F: Fn(SalaServiceClient<Channel>) -> Fut,
    Fut: std::future::Future<Output = Result<T, tonic::Status>>,
{
    const MAX_RETRIES: u32 = 2;

    for attempt in 0..MAX_RETRIES {
        let client = get_sala_client().await?;

        match operation(client).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                // Si es un error de conexión y no es el último intento, reconectar
                if is_connection_error(&e) && attempt < MAX_RETRIES - 1 {
                    reset_sala_client().await;
                    continue;
                }
                return Err(format!("Error gRPC: {}", e));
            }
        }
    }

    Err("Se alcanzó el número máximo de reintentos".to_string())
}

/// Determina si un error es de conexión (recuperable)
fn is_connection_error(status: &tonic::Status) -> bool {
    matches!(
        status.code(),
        tonic::Code::Unavailable
            | tonic::Code::Unknown
            | tonic::Code::Internal
            | tonic::Code::DeadlineExceeded
    )
}

// -------- API gRPC usando el cliente compartido con retry --------

// -------- API gRPC de Login --------

async fn login_usuario(email: String, password: String) -> Result<(String, UsuarioInfo), String> {
    let mut client = get_usuario_client().await?;

    let request: Request<LoginRequest> = Request::new(LoginRequest {
        email,
        password,
    });

    let response = client.login(request).await
        .map_err(|e| format!("Error al hacer login: {}", e))?;

    let login_response: LoginResponse = response.into_inner();
    let usuario = login_response.usuario.ok_or_else(|| "Respuesta de login sin usuario".to_string())?;

    let usuario_info = UsuarioInfo {
        id: usuario.id,
        nombre: usuario.nombre,
        email: usuario.email,
        rol: usuario.rol,
    };

    Ok((login_response.token, usuario_info))
}

// -------- API gRPC de Salas --------

async fn listar_salas() -> Result<Vec<SalaDto>, String> {
    let response = with_sala_retry(|mut client| async move {
        let request: Request<ListarSalasRequest> = Request::new(ListarSalasRequest {});
        client.listar_salas(request).await
    })
    .await?;

    Ok(response.into_inner().salas)
}

async fn crear_sala(nombre: String, capacidad: u32) -> Result<SalaDto, String> {
    let response = with_sala_retry(move |mut client| {
        let nombre = nombre.clone();
        async move {
            let request: Request<CrearSalaRequest> =
                Request::new(CrearSalaRequest { nombre, capacidad });
            client.crear_sala(request).await
        }
    })
    .await?;

    Ok(response.into_inner())
}

async fn activar_sala(id: String) -> Result<SalaDto, String> {
    let response = with_sala_retry(move |mut client| {
        let id = id.clone();
        async move {
            let request: Request<ActivarSalaRequest> = Request::new(ActivarSalaRequest { id });
            client.activar_sala(request).await
        }
    })
    .await?;

    Ok(response.into_inner())
}

async fn desactivar_sala(id: String) -> Result<SalaDto, String> {
    let response = with_sala_retry(move |mut client| {
        let id = id.clone();
        async move {
            let request: Request<DesactivarSalaRequest> =
                Request::new(DesactivarSalaRequest { id });
            client.desactivar_sala(request).await
        }
    })
    .await?;

    Ok(response.into_inner())
}
