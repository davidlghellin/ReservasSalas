use iced::widget::{button, column, container, row, scrollable, text, text_input, Column, Row};
use iced::{Alignment, Element, Length, Task, Theme};
use serde::{Deserialize, Serialize};

const BACKEND_URL: &str = "http://localhost:3000/api";

fn main() -> iced::Result {
    iced::application("Gestión de Salas - Iced", App::update, App::view)
        .theme(App::theme)
        .run_with(App::new)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SalaDto {
    id: String,
    nombre: String,
    capacidad: u32,
    activa: bool,
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
                self.mensaje = format!("✅ Sala '{}' creada correctamente", self.nuevo_nombre);
                self.nuevo_nombre.clear();
                self.nueva_capacidad = String::from("10");
                self.loading = false;
                Task::perform(listar_salas(), Message::SalasCargadas)
            }
            Message::SalaCreada(Err(e)) => {
                self.mensaje = format!("❌ Error al crear sala: {}", e);
                self.loading = false;
                Task::none()
            }
            Message::SalaActivada(Ok(_)) => {
                self.mensaje = "✅ Sala activada correctamente".to_string();
                self.loading = false;
                Task::perform(listar_salas(), Message::SalasCargadas)
            }
            Message::SalaActivada(Err(e)) => {
                self.mensaje = format!("❌ Error al activar sala: {}", e);
                self.loading = false;
                Task::none()
            }
            Message::SalaDesactivada(Ok(_)) => {
                self.mensaje = "✅ Sala desactivada correctamente".to_string();
                self.loading = false;
                Task::perform(listar_salas(), Message::SalasCargadas)
            }
            Message::SalaDesactivada(Err(e)) => {
                self.mensaje = format!("❌ Error al desactivar sala: {}", e);
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
                self.mensaje = "🔄 Actualizando...".to_string();
                Task::perform(listar_salas(), Message::SalasCargadas)
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let header = column![
            text("🏢 Gestión de Salas")
                .size(32)
                .width(Length::Fill)
                .center(),
            text("Sistema de reservas - Iced UI")
                .size(16)
                .width(Length::Fill)
                .center(),
        ]
        .spacing(5)
        .padding(20);

        let banner = container(
            text(format!("📋 Backend: {}", BACKEND_URL))
                .width(Length::Fill)
                .center(),
        )
        .padding(10)
        .width(Length::Fill)
        .center_x(Length::Fill);

        let mensaje_view = if !self.mensaje.is_empty() {
            container(
                text(&self.mensaje)
                    .width(Length::Fill)
                    .center(),
            )
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
            column![text("No hay salas registradas. Crea una nueva sala para comenzar.")
                .width(Length::Fill)
                .center()]
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
                                    row![
                                        text(&sala.nombre).size(18),
                                        badge,
                                    ]
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
        Theme::TokyoNight
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

async fn crear_sala(nombre: String, capacidad: u32) -> Result<SalaDto, String> {
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

async fn activar_sala(id: String) -> Result<SalaDto, String> {
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

async fn desactivar_sala(id: String) -> Result<SalaDto, String> {
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
