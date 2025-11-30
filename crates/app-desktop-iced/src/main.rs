#![allow(clippy::needless_return)]

use iced::widget::{button, column, container, row, scrollable, text, text_input, Column};
use iced::{Alignment, Element, Length, Task, Theme};

use tonic::transport::Channel;
use tonic::Request;

use salas_grpc::proto::sala_service_client::SalaServiceClient;
use salas_grpc::proto::{
    ActivarSalaRequest, CrearSalaRequest, DesactivarSalaRequest, ListarSalasRequest, SalaResponse,
};

use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(not(target_os = "macos"))]
use notify_rust::Notification;

const GRPC_URL: &str = "http://localhost:50051";

// Alias para simplificar el código de la UI
type SalaDto = SalaResponse;

// Tipo para el cliente compartido
type SharedClient = Arc<Mutex<Option<SalaServiceClient<Channel>>>>;

// Cliente gRPC compartido (una única conexión)
static GRPC_CLIENT: Lazy<SharedClient> = Lazy::new(|| Arc::new(Mutex::new(None)));

fn main() -> iced::Result {
    iced::application("Gestión de Salas - Iced (gRPC)", App::update, App::view)
        .theme(App::theme)
        .run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {
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

struct App {
    salas: Vec<SalaDto>,
    nuevo_nombre: String,
    nueva_capacidad: String,
    mensaje: String,
    loading: bool,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                salas: Vec::new(),
                nuevo_nombre: String::new(),
                nueva_capacidad: String::from("10"),
                mensaje: String::new(),
                loading: false,
            },
            Task::perform(listar_salas(), Message::SalasCargadas),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SalasCargadas(Ok(salas)) => {
                self.salas = salas;
                self.loading = false;
                Task::none()
            }
            Message::SalasCargadas(Err(e)) => {
                self.mensaje = format!("❌ Error al cargar salas: {}", e);
                self.loading = false;
                Task::none()
            }
            Message::SalaCreada(Ok(_)) => {
                let nombre = self.nuevo_nombre.clone();
                self.mensaje = format!("✅ Sala '{}' creada correctamente", nombre);
                mostrar_notificacion(
                    "✅ Sala creada",
                    &format!("La sala '{}' se creó correctamente", nombre),
                    TipoNotificacion::Exito,
                );
                self.nuevo_nombre.clear();
                self.nueva_capacidad = String::from("10");
                self.loading = false;
                Task::perform(listar_salas(), Message::SalasCargadas)
            }
            Message::SalaCreada(Err(e)) => {
                self.mensaje = format!("❌ Error al crear sala: {}", e);
                mostrar_notificacion("❌ Error", &e, TipoNotificacion::Error);
                self.loading = false;
                Task::none()
            }
            Message::SalaActivada(Ok(_)) => {
                self.mensaje = "✅ Sala activada correctamente".to_string();
                mostrar_notificacion(
                    "✅ Sala activada",
                    "La sala se activó correctamente",
                    TipoNotificacion::Exito,
                );
                self.loading = false;
                Task::perform(listar_salas(), Message::SalasCargadas)
            }
            Message::SalaActivada(Err(e)) => {
                self.mensaje = format!("❌ Error al activar sala: {}", e);
                mostrar_notificacion("❌ Error", &e, TipoNotificacion::Error);
                self.loading = false;
                Task::none()
            }
            Message::SalaDesactivada(Ok(_)) => {
                self.mensaje = "✅ Sala desactivada correctamente".to_string();
                mostrar_notificacion(
                    "✅ Sala desactivada",
                    "La sala se desactivó correctamente",
                    TipoNotificacion::Exito,
                );
                self.loading = false;
                Task::perform(listar_salas(), Message::SalasCargadas)
            }
            Message::SalaDesactivada(Err(e)) => {
                self.mensaje = format!("❌ Error al desactivar sala: {}", e);
                mostrar_notificacion("❌ Error", &e, TipoNotificacion::Error);
                self.loading = false;
                Task::none()
            }
            Message::NombreChanged(nombre) => {
                self.nuevo_nombre = nombre;
                Task::none()
            }
            Message::CapacidadChanged(capacidad) => {
                self.nueva_capacidad = capacidad;
                Task::none()
            }
            Message::CrearSala => {
                if self.nuevo_nombre.is_empty() {
                    self.mensaje = "❌ El nombre no puede estar vacío".to_string();
                    return Task::none();
                }

                let capacidad = match self.nueva_capacidad.parse::<u32>() {
                    Ok(c) if c > 0 => c,
                    _ => {
                        self.mensaje = "❌ La capacidad debe ser un número mayor que 0".to_string();
                        return Task::none();
                    }
                };

                self.loading = true;
                self.mensaje.clear();
                let nombre = self.nuevo_nombre.clone();
                Task::perform(crear_sala(nombre, capacidad), Message::SalaCreada)
            }
            Message::ActivarSala(id) => {
                self.loading = true;
                Task::perform(activar_sala(id), Message::SalaActivada)
            }
            Message::DesactivarSala(id) => {
                self.loading = true;
                Task::perform(desactivar_sala(id), Message::SalaDesactivada)
            }
            Message::ActualizarSalas => {
                self.loading = true;
                self.mensaje.clear(); // limpiamos mensaje al actualizar
                Task::perform(listar_salas(), Message::SalasCargadas)
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let header = column![
            text("🏢 Gestión de Salas")
                .size(32)
                .width(Length::Fill)
                .center(),
            text("Sistema de reservas - Iced UI (gRPC)")
                .size(16)
                .width(Length::Fill)
                .center(),
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

        let mensaje_view = if !self.mensaje.is_empty() {
            container(text(&self.mensaje).width(Length::Fill).center())
                .padding(15)
                .width(Length::Fill)
                .center_x(Length::Fill)
        } else {
            container(text(""))
        };

        let form = column![
            text("➕ Nueva Sala").size(20),
            row![
                text_input("Nombre de la sala", &self.nuevo_nombre)
                    .on_input(Message::NombreChanged)
                    .padding(10)
                    .width(Length::FillPortion(3)),
                text_input("Capacidad", &self.nueva_capacidad)
                    .on_input(Message::CapacidadChanged)
                    .padding(10)
                    .width(Length::FillPortion(1)),
                button(text(if self.loading {
                    "⏳ Creando..."
                } else {
                    "➕ Crear Sala"
                }))
                .on_press_maybe(if !self.loading {
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
            text(format!("📋 Lista de Salas ({})", self.salas.len())).size(20),
            button(text("🔄 Actualizar"))
                .on_press_maybe(if !self.loading {
                    Some(Message::ActualizarSalas)
                } else {
                    None
                })
                .padding(10),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .width(Length::Fill);

        let salas_list = if self.salas.is_empty() {
            column![
                text("No hay salas registradas. Crea una nueva sala para comenzar.")
                    .width(Length::Fill)
                    .center()
            ]
            .padding(40)
            .width(Length::Fill)
        } else {
            Column::with_children(
                self.salas
                    .iter()
                    .map(|sala| {
                        let badge = if sala.activa {
                            text("✅ Activa")
                        } else {
                            text("⏸️ Inactiva")
                        };

                        let action_button = if sala.activa {
                            button(text("⏸️ Desactivar"))
                                .on_press_maybe(if !self.loading {
                                    Some(Message::DesactivarSala(sala.id.clone()))
                                } else {
                                    None
                                })
                                .padding(8)
                        } else {
                            button(text("▶️ Activar"))
                                .on_press_maybe(if !self.loading {
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

/// Obtiene el cliente gRPC, creando una nueva conexión si es necesario
async fn get_client() -> Result<SalaServiceClient<Channel>, String> {
    let mut guard = GRPC_CLIENT.lock().await;

    if let Some(client) = guard.as_ref() {
        return Ok(client.clone());
    }

    // Si no hay cliente, conectar
    let client = connect_grpc().await?;
    *guard = Some(client.clone());
    Ok(client)
}

/// Resetea el cliente gRPC para forzar reconexión
async fn reset_client() {
    let mut guard = GRPC_CLIENT.lock().await;
    *guard = None;
}

fn grpc_url() -> String {
    std::env::var("GRPC_URL").unwrap_or_else(|_| GRPC_URL.to_string())
}

/// Conecta al servidor gRPC
async fn connect_grpc() -> Result<SalaServiceClient<Channel>, String> {
    SalaServiceClient::connect(grpc_url())
        .await
        .map_err(|e| format!("Error de conexión gRPC: {}", e))
}

/// Ejecuta una operación gRPC con reconexión automática en caso de fallo
async fn with_retry<F, Fut, T>(operation: F) -> Result<T, String>
where
    F: Fn(SalaServiceClient<Channel>) -> Fut,
    Fut: std::future::Future<Output = Result<T, tonic::Status>>,
{
    const MAX_RETRIES: u32 = 2;

    for attempt in 0..MAX_RETRIES {
        let client = get_client().await?;

        match operation(client).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                // Si es un error de conexión y no es el último intento, reconectar
                if is_connection_error(&e) && attempt < MAX_RETRIES - 1 {
                    reset_client().await;
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

async fn listar_salas() -> Result<Vec<SalaDto>, String> {
    let response = with_retry(|mut client| async move {
        let request: Request<ListarSalasRequest> = Request::new(ListarSalasRequest {});
        client.listar_salas(request).await
    })
    .await?;

    Ok(response.into_inner().salas)
}

async fn crear_sala(nombre: String, capacidad: u32) -> Result<SalaDto, String> {
    let response = with_retry(move |mut client| {
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
    let response = with_retry(move |mut client| {
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
    let response = with_retry(move |mut client| {
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
