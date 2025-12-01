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

use reservas_grpc::proto::reserva_service_client::ReservaServiceClient;
use reservas_grpc::proto::{
    CancelarReservaRequest, CrearReservaRequest, ListarReservasRequest, Reserva as ProtoReserva,
};

use chrono::{Duration as ChronoDuration, Local};

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

// Tipo para el cliente compartido de reservas
type SharedReservaClient = Arc<Mutex<Option<ReservaServiceClient<Channel>>>>;

// Cliente gRPC compartido de salas (una única conexión)
static GRPC_SALA_CLIENT: Lazy<SharedSalaClient> = Lazy::new(|| Arc::new(Mutex::new(None)));

// Cliente gRPC compartido de usuarios (una única conexión)
static GRPC_USUARIO_CLIENT: Lazy<SharedUsuarioClient> = Lazy::new(|| Arc::new(Mutex::new(None)));

// Cliente gRPC compartido de reservas (una única conexión)
static GRPC_RESERVA_CLIENT: Lazy<SharedReservaClient> = Lazy::new(|| Arc::new(Mutex::new(None)));

// Token JWT almacenado
static JWT_TOKEN: Lazy<Arc<Mutex<Option<String>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

fn main() -> iced::Result {
    iced::application("Gestión de Salas - Iced (gRPC)", App::update, App::view)
        .theme(App::theme)
        .run_with(App::new)
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum Message {
    // Mensajes de login
    EmailChanged(String),
    PasswordChanged(String),
    Login,
    LoginExitoso(String, UsuarioInfo),
    LoginError(String),
    Logout,

    // Mensajes de navegación
    CambiarTab(Tab),

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

    // Mensajes de reservas
    ReservasCargadas(Result<Vec<ProtoReserva>, String>),
    ReservaCreada(Result<ProtoReserva, String>),
    ReservaCancelada(Result<ProtoReserva, String>),
    DisponibilidadVerificada(Result<bool, String>),
    SalaSeleccionadaChanged(String),
    FechaInicioChanged(String),
    FechaFinChanged(String),
    CrearReserva,
    CancelarReserva(String),
    ActualizarReservas,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Salas,
    Reservas,
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
        usuario: Box<UsuarioInfo>,
        tab_actual: Tab,

        // Estado de salas
        salas: Vec<SalaDto>,
        nuevo_nombre: String,
        nueva_capacidad: String,

        // Estado de reservas
        reservas: Vec<ProtoReserva>,
        sala_seleccionada: String,
        fecha_inicio: String,
        fecha_fin: String,

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
                if let AppState::Login {
                    email,
                    password,
                    loading,
                    error,
                } = &mut self.state
                {
                    if email.is_empty() || password.is_empty() {
                        *error = "Email y contraseña son requeridos".to_string();
                        return Task::none();
                    }
                    *loading = true;
                    *error = String::new();
                    let email = email.clone();
                    let password = password.clone();
                    Task::perform(login_usuario(email, password), |result| match result {
                        Ok((token, usuario)) => Message::LoginExitoso(token, usuario),
                        Err(e) => Message::LoginError(e),
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

                // Fecha por defecto (mañana a las 10:00)
                let manana = Local::now() + ChronoDuration::days(1);
                let fecha_inicio_default = manana.format("%Y-%m-%dT10:00").to_string();
                let fecha_fin_default = manana.format("%Y-%m-%dT12:00").to_string();

                // Cambiar a estado autenticado
                self.state = AppState::Authenticated {
                    usuario: Box::new(usuario),
                    tab_actual: Tab::Salas,
                    salas: Vec::new(),
                    nuevo_nombre: String::new(),
                    nueva_capacidad: String::from("10"),
                    reservas: Vec::new(),
                    sala_seleccionada: String::new(),
                    fecha_inicio: fecha_inicio_default,
                    fecha_fin: fecha_fin_default,
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
                if let AppState::Login {
                    error: e, loading, ..
                } = &mut self.state
                {
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
                if let AppState::Authenticated {
                    salas: s, loading, ..
                } = &mut self.state
                {
                    *s = salas;
                    *loading = false;
                }
                Task::none()
            }
            Message::SalasCargadas(Err(e)) => {
                if let AppState::Authenticated {
                    mensaje, loading, ..
                } = &mut self.state
                {
                    *mensaje = format!("❌ Error al cargar salas: {}", e);
                    *loading = false;
                }
                Task::none()
            }
            Message::SalaCreada(Ok(_)) => {
                if let AppState::Authenticated {
                    nuevo_nombre,
                    nueva_capacidad,
                    mensaje,
                    loading,
                    ..
                } = &mut self.state
                {
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
                if let AppState::Authenticated {
                    mensaje, loading, ..
                } = &mut self.state
                {
                    *mensaje = format!("❌ Error al crear sala: {}", e);
                    *loading = false;
                }
                mostrar_notificacion("❌ Error", &e, TipoNotificacion::Error);
                Task::none()
            }
            Message::SalaActivada(Ok(_)) => {
                if let AppState::Authenticated {
                    mensaje, loading, ..
                } = &mut self.state
                {
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
                if let AppState::Authenticated {
                    mensaje, loading, ..
                } = &mut self.state
                {
                    *mensaje = format!("❌ Error al activar sala: {}", e);
                    *loading = false;
                }
                mostrar_notificacion("❌ Error", &e, TipoNotificacion::Error);
                Task::none()
            }
            Message::SalaDesactivada(Ok(_)) => {
                if let AppState::Authenticated {
                    mensaje, loading, ..
                } = &mut self.state
                {
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
                if let AppState::Authenticated {
                    mensaje, loading, ..
                } = &mut self.state
                {
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
                if let AppState::Authenticated {
                    nueva_capacidad, ..
                } = &mut self.state
                {
                    *nueva_capacidad = capacidad;
                }
                Task::none()
            }
            Message::CrearSala => {
                if let AppState::Authenticated {
                    nuevo_nombre,
                    nueva_capacidad,
                    mensaje,
                    loading,
                    ..
                } = &mut self.state
                {
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
                if let AppState::Authenticated {
                    mensaje, loading, ..
                } = &mut self.state
                {
                    *loading = true;
                    mensaje.clear();
                }
                Task::perform(listar_salas(), Message::SalasCargadas)
            }

            // Mensajes de navegación
            Message::CambiarTab(tab) => {
                if let AppState::Authenticated {
                    tab_actual,
                    loading,
                    ..
                } = &mut self.state
                {
                    *tab_actual = tab;
                    *loading = true;
                }
                match tab {
                    Tab::Salas => Task::perform(listar_salas(), Message::SalasCargadas),
                    Tab::Reservas => {
                        // Cargar tanto salas como reservas cuando se cambia a Reservas
                        Task::batch(vec![
                            Task::perform(listar_salas(), Message::SalasCargadas),
                            Task::perform(listar_reservas(), Message::ReservasCargadas),
                        ])
                    }
                }
            }

            // Mensajes de reservas
            Message::ReservasCargadas(Ok(reservas)) => {
                if let AppState::Authenticated {
                    reservas: r,
                    loading,
                    ..
                } = &mut self.state
                {
                    *r = reservas;
                    *loading = false;
                }
                Task::none()
            }
            Message::ReservasCargadas(Err(e)) => {
                if let AppState::Authenticated {
                    mensaje, loading, ..
                } = &mut self.state
                {
                    *mensaje = format!("❌ Error al cargar reservas: {}", e);
                    *loading = false;
                }
                Task::none()
            }
            Message::ReservaCreada(Ok(_)) => {
                if let AppState::Authenticated {
                    mensaje,
                    loading,
                    sala_seleccionada,
                    ..
                } = &mut self.state
                {
                    *mensaje = "✅ Reserva creada correctamente".to_string();
                    mostrar_notificacion(
                        "✅ Reserva creada",
                        "La reserva se creó correctamente",
                        TipoNotificacion::Exito,
                    );
                    sala_seleccionada.clear();
                    *loading = false;
                }
                Task::perform(listar_reservas(), Message::ReservasCargadas)
            }
            Message::ReservaCreada(Err(e)) => {
                if let AppState::Authenticated {
                    mensaje, loading, ..
                } = &mut self.state
                {
                    *mensaje = format!("❌ Error al crear reserva: {}", e);
                    *loading = false;
                }
                mostrar_notificacion("❌ Error", &e, TipoNotificacion::Error);
                Task::none()
            }
            Message::ReservaCancelada(Ok(_)) => {
                if let AppState::Authenticated {
                    mensaje, loading, ..
                } = &mut self.state
                {
                    *mensaje = "✅ Reserva cancelada correctamente".to_string();
                    *loading = false;
                }
                mostrar_notificacion(
                    "✅ Reserva cancelada",
                    "La reserva se canceló correctamente",
                    TipoNotificacion::Exito,
                );
                Task::perform(listar_reservas(), Message::ReservasCargadas)
            }
            Message::ReservaCancelada(Err(e)) => {
                if let AppState::Authenticated {
                    mensaje, loading, ..
                } = &mut self.state
                {
                    *mensaje = format!("❌ Error al cancelar reserva: {}", e);
                    *loading = false;
                }
                mostrar_notificacion("❌ Error", &e, TipoNotificacion::Error);
                Task::none()
            }
            Message::DisponibilidadVerificada(Ok(disponible)) => {
                if let AppState::Authenticated { mensaje, .. } = &mut self.state {
                    *mensaje = if disponible {
                        "✅ Sala disponible en ese horario".to_string()
                    } else {
                        "❌ Sala NO disponible - hay conflictos".to_string()
                    };
                }
                Task::none()
            }
            Message::DisponibilidadVerificada(Err(e)) => {
                if let AppState::Authenticated { mensaje, .. } = &mut self.state {
                    *mensaje = format!("❌ Error al verificar disponibilidad: {}", e);
                }
                Task::none()
            }
            Message::SalaSeleccionadaChanged(sala_id) => {
                if let AppState::Authenticated {
                    sala_seleccionada, ..
                } = &mut self.state
                {
                    *sala_seleccionada = sala_id;
                }
                Task::none()
            }
            Message::FechaInicioChanged(fecha) => {
                if let AppState::Authenticated { fecha_inicio, .. } = &mut self.state {
                    *fecha_inicio = fecha;
                }
                Task::none()
            }
            Message::FechaFinChanged(fecha) => {
                if let AppState::Authenticated { fecha_fin, .. } = &mut self.state {
                    *fecha_fin = fecha;
                }
                Task::none()
            }
            Message::CrearReserva => {
                if let AppState::Authenticated {
                    usuario,
                    sala_seleccionada,
                    fecha_inicio,
                    fecha_fin,
                    mensaje,
                    loading,
                    ..
                } = &mut self.state
                {
                    if sala_seleccionada.is_empty() {
                        *mensaje = "❌ Debes seleccionar una sala".to_string();
                        return Task::none();
                    }

                    *loading = true;
                    mensaje.clear();

                    let sala_id = sala_seleccionada.clone();
                    let usuario_id = usuario.id.clone();
                    let inicio = fecha_inicio.clone();
                    let fin = fecha_fin.clone();

                    Task::perform(
                        crear_reserva(sala_id, usuario_id, inicio, fin),
                        Message::ReservaCreada,
                    )
                } else {
                    Task::none()
                }
            }
            Message::CancelarReserva(id) => {
                if let AppState::Authenticated { loading, .. } = &mut self.state {
                    *loading = true;
                }
                Task::perform(cancelar_reserva(id), Message::ReservaCancelada)
            }
            Message::ActualizarReservas => {
                if let AppState::Authenticated {
                    mensaje, loading, ..
                } = &mut self.state
                {
                    *loading = true;
                    mensaje.clear();
                }
                Task::perform(listar_reservas(), Message::ReservasCargadas)
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.state {
            AppState::Login {
                email,
                password,
                error,
                loading,
            } => self.view_login(email, password, error, *loading),
            AppState::Authenticated {
                usuario,
                tab_actual,
                salas,
                nuevo_nombre,
                nueva_capacidad,
                reservas,
                sala_seleccionada,
                fecha_inicio,
                fecha_fin,
                mensaje,
                loading,
            } => self.view_main(
                usuario,
                *tab_actual,
                salas,
                nuevo_nombre,
                nueva_capacidad,
                reservas,
                sala_seleccionada,
                fecha_inicio,
                fecha_fin,
                mensaje,
                *loading,
            ),
        }
    }

    fn view_login<'a>(
        &'a self,
        email: &'a str,
        password: &'a str,
        error: &'a str,
        loading: bool,
    ) -> Element<'a, Message> {
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
            container(text(error).size(14).width(Length::Fill).center()).padding(10)
        } else {
            container(text(""))
        };

        let form = column![
            text("Email:").size(16),
            text_input("usuario@ejemplo.com", email)
                .on_input(Message::EmailChanged)
                .padding(10)
                .width(Length::Fill),
            text("Contraseña:").size(16),
            text_input("", password)
                .on_input(Message::PasswordChanged)
                .padding(10)
                .width(Length::Fill),
            button(text(if loading {
                "⏳ Iniciando sesión..."
            } else {
                "🚀 Iniciar Sesión"
            }))
            .on_press_maybe(if !loading { Some(Message::Login) } else { None })
            .padding(15)
            .width(Length::Fill),
        ]
        .spacing(15)
        .padding(40)
        .max_width(400);

        let content = column![title, error_message, form,]
            .spacing(20)
            .align_x(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    #[allow(clippy::too_many_arguments)]
    fn view_main<'a>(
        &'a self,
        usuario: &'a UsuarioInfo,
        tab_actual: Tab,
        salas: &'a [SalaDto],
        nuevo_nombre: &'a str,
        nueva_capacidad: &'a str,
        reservas: &'a [ProtoReserva],
        sala_seleccionada: &'a str,
        fecha_inicio: &'a str,
        fecha_fin: &'a str,
        mensaje: &'a str,
        loading: bool,
    ) -> Element<'a, Message> {
        let header = column![row![
            column![
                text("🏢 Gestión de Salas").size(32).width(Length::Fill),
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
        .align_y(Alignment::Center),]
        .spacing(5)
        .padding(20);

        // Tabs de navegación
        let tabs = row![
            button(text(if tab_actual == Tab::Salas {
                "🏢 Salas ✓"
            } else {
                "🏢 Salas"
            }))
            .on_press(Message::CambiarTab(Tab::Salas))
            .padding(15),
            button(text(if tab_actual == Tab::Reservas {
                "📅 Reservas ✓"
            } else {
                "📅 Reservas"
            }))
            .on_press(Message::CambiarTab(Tab::Reservas))
            .padding(15),
        ]
        .spacing(10)
        .padding(10);

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

        // Contenido según el tab actual
        let contenido = match tab_actual {
            Tab::Salas => self.view_salas_tab(salas, nuevo_nombre, nueva_capacidad, loading),
            Tab::Reservas => self.view_reservas_tab(
                reservas,
                salas,
                sala_seleccionada,
                fecha_inicio,
                fecha_fin,
                loading,
            ),
        };

        let content = column![header, tabs, banner, mensaje_view, contenido]
            .spacing(20)
            .padding(20)
            .width(Length::Fill);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .into()
    }

    fn view_salas_tab<'a>(
        &'a self,
        salas: &'a [SalaDto],
        nuevo_nombre: &'a str,
        nueva_capacidad: &'a str,
        loading: bool,
    ) -> Element<'a, Message> {
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

        column![form, salas_container]
            .spacing(20)
            .width(Length::Fill)
            .into()
    }

    fn view_reservas_tab<'a>(
        &'a self,
        reservas: &'a [ProtoReserva],
        salas: &'a [SalaDto],
        sala_seleccionada: &'a str,
        fecha_inicio: &'a str,
        fecha_fin: &'a str,
        loading: bool,
    ) -> Element<'a, Message> {
        // Formulario para crear reserva
        let form = column![
            text("➕ Nueva Reserva").size(20),
            row![
                column![
                    text("Sala:").size(14),
                    scrollable(
                        Column::with_children(
                            salas
                                .iter()
                                .filter(|s| s.activa)
                                .map(|sala| {
                                    button(text(format!(
                                        "{} {}",
                                        if sala_seleccionada == sala.id.as_str() {
                                            "✓"
                                        } else {
                                            "○"
                                        },
                                        sala.nombre
                                    )))
                                    .on_press(Message::SalaSeleccionadaChanged(sala.id.clone()))
                                    .padding(8)
                                    .width(Length::Fill)
                                    .into()
                                })
                                .collect::<Vec<_>>(),
                        )
                        .spacing(5)
                    )
                    .height(Length::Fixed(150.0)),
                ]
                .spacing(5)
                .width(Length::FillPortion(2)),
                column![
                    text("Fecha Inicio:").size(14),
                    text_input("YYYY-MM-DDTHH:MM", fecha_inicio)
                        .on_input(Message::FechaInicioChanged)
                        .padding(10),
                    text("Fecha Fin:").size(14),
                    text_input("YYYY-MM-DDTHH:MM", fecha_fin)
                        .on_input(Message::FechaFinChanged)
                        .padding(10),
                    button(text(if loading {
                        "⏳ Creando..."
                    } else {
                        "➕ Crear Reserva"
                    }))
                    .on_press_maybe(if !loading {
                        Some(Message::CrearReserva)
                    } else {
                        None
                    })
                    .padding(15)
                    .width(Length::Fill),
                ]
                .spacing(10)
                .width(Length::FillPortion(3)),
            ]
            .spacing(20)
            .align_y(Alignment::Center),
        ]
        .spacing(15)
        .padding(20)
        .width(Length::Fill);

        // Lista de reservas
        let reservas_header = row![
            text(format!("📋 Mis Reservas ({})", reservas.len())).size(20),
            button(text("🔄 Actualizar"))
                .on_press_maybe(if !loading {
                    Some(Message::ActualizarReservas)
                } else {
                    None
                })
                .padding(10),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .width(Length::Fill);

        let reservas_list = if reservas.is_empty() {
            column![
                text("No tienes reservas. Crea una nueva reserva para comenzar.")
                    .width(Length::Fill)
                    .center()
            ]
            .padding(40)
            .width(Length::Fill)
        } else {
            Column::with_children(
                reservas
                    .iter()
                    .map(|reserva| {
                        // Encontrar el nombre de la sala
                        let nombre_sala = salas
                            .iter()
                            .find(|s| s.id == reserva.sala_id)
                            .map(|s| s.nombre.clone())
                            .unwrap_or_else(|| "Sala desconocida".to_string());

                        // Formatear las fechas
                        let fecha_inicio_formatted =
                            reserva.fecha_inicio.split('T').collect::<Vec<_>>();
                        let fecha_fin_formatted = reserva.fecha_fin.split('T').collect::<Vec<_>>();

                        let estado_emoji = match reserva.estado {
                            0 => "✅", // Activa
                            1 => "❌", // Cancelada
                            2 => "✔️", // Completada
                            _ => "❓",
                        };

                        let estado_text = match reserva.estado {
                            0 => "Activa",
                            1 => "Cancelada",
                            2 => "Completada",
                            _ => "Desconocida",
                        };

                        let cancel_button = if reserva.estado == 0 {
                            button(text("❌ Cancelar"))
                                .on_press_maybe(if !loading {
                                    Some(Message::CancelarReserva(reserva.id.clone()))
                                } else {
                                    None
                                })
                                .padding(8)
                        } else {
                            button(text(""))
                        };

                        container(
                            row![
                                column![
                                    row![
                                        text(nombre_sala).size(18),
                                        text(format!("{} {}", estado_emoji, estado_text)),
                                    ]
                                    .spacing(10)
                                    .align_y(Alignment::Center),
                                    text(format!(
                                        "📅 {} {} - {} {}",
                                        fecha_inicio_formatted.first().unwrap_or(&""),
                                        fecha_inicio_formatted.get(1).unwrap_or(&""),
                                        fecha_fin_formatted.first().unwrap_or(&""),
                                        fecha_fin_formatted.get(1).unwrap_or(&""),
                                    )),
                                    text(format!("ID: {}", reserva.id)).size(12),
                                ]
                                .spacing(8)
                                .width(Length::Fill),
                                cancel_button,
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

        let reservas_container = column![reservas_header, scrollable(reservas_list)]
            .spacing(15)
            .padding(20)
            .width(Length::Fill);

        column![form, reservas_container]
            .spacing(20)
            .width(Length::Fill)
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
            TipoNotificacion::Exito => "Glass",  // Sonido de éxito
            TipoNotificacion::Error => "Basso",  // Sonido de error
            TipoNotificacion::Info => "default", // Sonido por defecto
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

#[allow(dead_code)]
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

// -------- Helpers para JWT --------

/// Obtiene el token JWT almacenado
async fn get_jwt_token() -> Option<String> {
    let guard = JWT_TOKEN.lock().await;
    guard.clone()
}

/// Agrega el token JWT al header de autorización de una request gRPC
fn add_auth_token<T>(request: &mut Request<T>, token: &str) -> Result<(), String> {
    use tonic::metadata::MetadataValue;

    let auth_value = MetadataValue::try_from(format!("Bearer {}", token))
        .map_err(|e| format!("Error al crear header de autorización: {}", e))?;

    request.metadata_mut().insert("authorization", auth_value);
    Ok(())
}

// -------- API gRPC de Login --------

async fn login_usuario(email: String, password: String) -> Result<(String, UsuarioInfo), String> {
    let mut client = get_usuario_client().await?;

    let request: Request<LoginRequest> = Request::new(LoginRequest { email, password });

    let response = client
        .login(request)
        .await
        .map_err(|e| format!("Error al hacer login: {}", e))?;

    let login_response: LoginResponse = response.into_inner();
    let usuario = login_response
        .usuario
        .ok_or_else(|| "Respuesta de login sin usuario".to_string())?;

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
        let mut request: Request<ListarSalasRequest> = Request::new(ListarSalasRequest {});

        // Agregar token JWT si existe
        if let Some(token) = get_jwt_token().await {
            add_auth_token(&mut request, &token).map_err(tonic::Status::internal)?;
        }

        client.listar_salas(request).await
    })
    .await?;

    Ok(response.into_inner().salas)
}

async fn crear_sala(nombre: String, capacidad: u32) -> Result<SalaDto, String> {
    let response = with_sala_retry(move |mut client| {
        let nombre = nombre.clone();
        async move {
            let mut request: Request<CrearSalaRequest> =
                Request::new(CrearSalaRequest { nombre, capacidad });

            // Agregar token JWT si existe
            if let Some(token) = get_jwt_token().await {
                add_auth_token(&mut request, &token).map_err(tonic::Status::internal)?;
            }

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
            let mut request: Request<ActivarSalaRequest> = Request::new(ActivarSalaRequest { id });

            // Agregar token JWT si existe
            if let Some(token) = get_jwt_token().await {
                add_auth_token(&mut request, &token).map_err(tonic::Status::internal)?;
            }

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
            let mut request: Request<DesactivarSalaRequest> =
                Request::new(DesactivarSalaRequest { id });

            // Agregar token JWT si existe
            if let Some(token) = get_jwt_token().await {
                add_auth_token(&mut request, &token).map_err(tonic::Status::internal)?;
            }

            client.desactivar_sala(request).await
        }
    })
    .await?;

    Ok(response.into_inner())
}

// -------- API gRPC de Reservas --------

/// Obtiene el cliente gRPC de reservas, creando una nueva conexión si es necesario
async fn get_reserva_client() -> Result<ReservaServiceClient<Channel>, String> {
    let mut guard = GRPC_RESERVA_CLIENT.lock().await;

    if let Some(client) = guard.as_ref() {
        return Ok(client.clone());
    }

    // Si no hay cliente, conectar
    let client = connect_reserva_grpc().await?;
    *guard = Some(client.clone());
    Ok(client)
}

/// Conecta al servidor gRPC de reservas
async fn connect_reserva_grpc() -> Result<ReservaServiceClient<Channel>, String> {
    ReservaServiceClient::connect(grpc_url())
        .await
        .map_err(|e| format!("Error de conexión gRPC reservas: {}", e))
}

async fn listar_reservas() -> Result<Vec<ProtoReserva>, String> {
    let mut client = get_reserva_client().await?;

    let mut request: Request<ListarReservasRequest> = Request::new(ListarReservasRequest {});

    // Agregar token JWT si existe
    if let Some(token) = get_jwt_token().await {
        add_auth_token(&mut request, &token)
            .map_err(|e| format!("Error al agregar token: {}", e))?;
    }

    let response = client
        .listar_reservas(request)
        .await
        .map_err(|e| format!("Error al listar reservas: {}", e))?;

    Ok(response.into_inner().reservas)
}

async fn crear_reserva(
    sala_id: String,
    usuario_id: String,
    fecha_inicio: String,
    fecha_fin: String,
) -> Result<ProtoReserva, String> {
    let mut client = get_reserva_client().await?;

    // Convertir fechas al formato RFC3339
    let fecha_inicio_rfc = format!("{}:00Z", fecha_inicio.replace(" ", "T"));
    let fecha_fin_rfc = format!("{}:00Z", fecha_fin.replace(" ", "T"));

    let mut request: Request<CrearReservaRequest> = Request::new(CrearReservaRequest {
        sala_id,
        usuario_id,
        fecha_inicio: fecha_inicio_rfc,
        fecha_fin: fecha_fin_rfc,
    });

    // Agregar token JWT si existe
    if let Some(token) = get_jwt_token().await {
        add_auth_token(&mut request, &token)
            .map_err(|e| format!("Error al agregar token: {}", e))?;
    }

    let response = client
        .crear_reserva(request)
        .await
        .map_err(|e| format!("Error al crear reserva: {}", e))?;

    response
        .into_inner()
        .reserva
        .ok_or_else(|| "Respuesta sin reserva".to_string())
}

async fn cancelar_reserva(id: String) -> Result<ProtoReserva, String> {
    let mut client = get_reserva_client().await?;

    let mut request: Request<CancelarReservaRequest> = Request::new(CancelarReservaRequest { id });

    // Agregar token JWT si existe
    if let Some(token) = get_jwt_token().await {
        add_auth_token(&mut request, &token)
            .map_err(|e| format!("Error al agregar token: {}", e))?;
    }

    let response = client
        .cancelar_reserva(request)
        .await
        .map_err(|e| format!("Error al cancelar reserva: {}", e))?;

    response
        .into_inner()
        .reserva
        .ok_or_else(|| "Respuesta sin reserva".to_string())
}
