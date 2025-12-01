use crossterm::{event, terminal, ExecutableCommand};
use ratatui::{prelude::*, widgets::*};
use tonic::{metadata::MetadataValue, Request};

use salas_grpc::proto::{sala_service_client::SalaServiceClient, ListarSalasRequest};
use usuarios_grpc::proto::{usuario_service_client::UsuarioServiceClient, LoginRequest};

const GRPC_URL: &str = "http://localhost:50051";

struct Sala {
    id: String,
    nombre: String,
    capacidad: u32,
    activa: bool,
}

struct UsuarioInfo {
    id: String,
    nombre: String,
    email: String,
    rol: String,
}

enum LoginField {
    Email,
    Password,
}

enum AppState {
    Login {
        email: String,
        password: String,
        active_field: LoginField,
        error: Option<String>,
    },
    Authenticated {
        usuario: UsuarioInfo,
        token: String,
    },
    Menu {
        token: String,
    },
    ListarSalas {
        salas: Vec<Sala>,
        token: String,
    },
    Error {
        message: String,
        token: Option<String>,
    },
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    terminal::enable_raw_mode()?;
    std::io::stdout().execute(terminal::EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    let mut state = AppState::Login {
        email: String::new(),
        password: String::new(),
        active_field: LoginField::Email,
        error: None,
    };

    loop {
        let _ = terminal.draw(|f| match &state {
            AppState::Login {
                email,
                password,
                active_field,
                error,
            } => {
                render_login_screen(f, email, password, active_field, error.as_deref());
            }
            AppState::Authenticated { usuario, .. } => {
                render_welcome_screen(f, usuario);
            }
            AppState::Menu { .. } => {
                let block = Block::default().title("Menú").borders(Borders::ALL);
                let items = ["1. Listar Salas", "q. Salir"];
                let paragraph = Paragraph::new(items.join("\n"))
                    .block(block)
                    .alignment(Alignment::Left);
                f.render_widget(paragraph, f.area());
            }
            AppState::ListarSalas { salas, .. } => {
                let rows: Vec<Row> = salas
                    .iter()
                    .map(|s| {
                        Row::new(vec![
                            s.id.clone(),
                            s.nombre.clone(),
                            s.capacidad.to_string(),
                            s.activa.to_string(),
                        ])
                    })
                    .collect();
                let widths = [
                    Constraint::Length(5),
                    Constraint::Length(10),
                    Constraint::Length(10),
                    Constraint::Length(10),
                ];
                let table = Table::new(rows, widths)
                    .header(
                        Row::new(vec!["ID", "Nombre", "Capacidad", "Activa"])
                            .style(Style::default().fg(Color::Yellow)),
                    )
                    .block(Block::default().title("Salas").borders(Borders::ALL));
                f.render_widget(table, f.area());
            }
            AppState::Error { message, .. } => {
                let paragraph = Paragraph::new(message.clone())
                    .block(Block::default().title("Error").borders(Borders::ALL))
                    .alignment(Alignment::Center);
                f.render_widget(paragraph, f.area());
            }
        });

        if event::poll(std::time::Duration::from_millis(100))? {
            if let event::Event::Key(key) = event::read()? {
                match &mut state {
                    AppState::Login {
                        email,
                        password,
                        active_field,
                        error,
                    } => match key.code {
                        event::KeyCode::Tab => {
                            *active_field = match active_field {
                                LoginField::Email => LoginField::Password,
                                LoginField::Password => LoginField::Email,
                            };
                            *error = None;
                        }
                        event::KeyCode::Enter => {
                            if email.is_empty() || password.is_empty() {
                                *error = Some("Email y contraseña son requeridos".to_string());
                            } else {
                                match login_usuario(email.clone(), password.clone()).await {
                                    Ok((usuario, token)) => {
                                        state = AppState::Authenticated { usuario, token };
                                    }
                                    Err(e) => {
                                        *error = Some(e);
                                    }
                                }
                            }
                        }
                        event::KeyCode::Backspace => {
                            match active_field {
                                LoginField::Email => {
                                    email.pop();
                                }
                                LoginField::Password => {
                                    password.pop();
                                }
                            }
                            *error = None;
                        }
                        event::KeyCode::Char(c) => {
                            match active_field {
                                LoginField::Email => email.push(c),
                                LoginField::Password => password.push(c),
                            }
                            *error = None;
                        }
                        event::KeyCode::Esc => {
                            break;
                        }
                        _ => {}
                    },
                    AppState::Authenticated { usuario: _, token } => match key.code {
                        event::KeyCode::Enter | event::KeyCode::Char(' ') => {
                            state = AppState::Menu {
                                token: token.clone(),
                            };
                        }
                        event::KeyCode::Esc => {
                            break;
                        }
                        _ => {}
                    },
                    AppState::Menu { token } => match key.code {
                        event::KeyCode::Char('1') => match listar_salas(token).await {
                            Ok(salas) => {
                                state = AppState::ListarSalas {
                                    salas,
                                    token: token.clone(),
                                };
                            }
                            Err(e) => {
                                state = AppState::Error {
                                    message: e,
                                    token: Some(token.clone()),
                                };
                            }
                        },
                        event::KeyCode::Char('q') => {
                            break;
                        }
                        _ => {}
                    },
                    AppState::ListarSalas { token, .. } => match key.code {
                        event::KeyCode::Char('q') | event::KeyCode::Esc => {
                            state = AppState::Menu {
                                token: token.clone(),
                            };
                        }
                        _ => {}
                    },
                    AppState::Error { token, .. } => match key.code {
                        event::KeyCode::Char('q') | event::KeyCode::Esc => {
                            if let Some(tok) = token {
                                state = AppState::Menu { token: tok.clone() };
                            } else {
                                break;
                            }
                        }
                        _ => {}
                    },
                }
            }
        }
    }

    terminal::disable_raw_mode()?;
    std::io::stdout().execute(terminal::LeaveAlternateScreen)?;
    Ok(())
}

fn render_login_screen(
    f: &mut Frame,
    email: &str,
    password: &str,
    active_field: &LoginField,
    error: Option<&str>,
) {
    let size = f.area();
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(size);

    // Título
    let title = Paragraph::new("🔐 Iniciar Sesión")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Sistema de Gestión de Salas"),
        )
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(title, area[0]);

    // Email field
    let email_style = match active_field {
        LoginField::Email => Style::default().fg(Color::Yellow),
        _ => Style::default().fg(Color::White),
    };
    let email_input = Paragraph::new(format!("Email: {}", email))
        .block(Block::default().borders(Borders::ALL).title("Email"))
        .style(email_style);
    f.render_widget(email_input, area[1]);
    if matches!(active_field, LoginField::Email) {
        let x = area[1].x + 7 + email.len() as u16;
        let y = area[1].y + 1;
        f.set_cursor_position(Position { x, y });
    }

    // Password field (mostrar asteriscos)
    let password_display = "*".repeat(password.len());
    let password_style = match active_field {
        LoginField::Password => Style::default().fg(Color::Yellow),
        _ => Style::default().fg(Color::White),
    };
    let password_input = Paragraph::new(format!("Password: {}", password_display))
        .block(Block::default().borders(Borders::ALL).title("Contraseña"))
        .style(password_style);
    f.render_widget(password_input, area[2]);
    if matches!(active_field, LoginField::Password) {
        let x = area[2].x + 10 + password.len() as u16;
        let y = area[2].y + 1;
        f.set_cursor_position(Position { x, y });
    }

    // Instrucciones
    let instructions = Paragraph::new("Tab: Cambiar campo | Enter: Login | Esc: Salir")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(instructions, area[3]);

    // Error message
    if let Some(err) = error {
        let error_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3)])
            .split(area[4]);
        let error_para = Paragraph::new(format!("❌ Error: {}", err))
            .block(Block::default().borders(Borders::ALL).title("Error"))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(error_para, error_area[0]);
    }
}

fn render_welcome_screen(f: &mut Frame, usuario: &UsuarioInfo) {
    let size = f.area();
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(size);

    let title = Paragraph::new("✅ Login Exitoso")
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(title, area[0]);

    let info = format!(
        "👤 Usuario: {}\n📧 Email: {}\n🎫 Rol: {}",
        usuario.nombre, usuario.email, usuario.rol
    );
    let info_para = Paragraph::new(info)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Información de Usuario"),
        )
        .alignment(Alignment::Center);
    f.render_widget(info_para, area[1]);

    let instructions = Paragraph::new("Presiona Enter o Espacio para continuar")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(instructions, area[2]);
}

async fn login_usuario(email: String, password: String) -> Result<(UsuarioInfo, String), String> {
    let mut client = UsuarioServiceClient::connect(GRPC_URL)
        .await
        .map_err(|e| format!("Error de conexión gRPC: {}", e))?;

    let request = Request::new(LoginRequest {
        email: email.clone(),
        password,
    });

    let response = client
        .login(request)
        .await
        .map_err(|e| format!("Error al hacer login: {}", e))?;

    let login_response = response.into_inner();
    let usuario_proto = login_response
        .usuario
        .ok_or_else(|| "Respuesta sin usuario".to_string())?;

    let usuario = UsuarioInfo {
        id: usuario_proto.id,
        nombre: usuario_proto.nombre,
        email: usuario_proto.email,
        rol: usuario_proto.rol,
    };

    Ok((usuario, login_response.token))
}

async fn listar_salas(token: &str) -> Result<Vec<Sala>, String> {
    let mut client = SalaServiceClient::connect(GRPC_URL)
        .await
        .map_err(|e| format!("Error de conexión gRPC: {}", e))?;

    let mut request = Request::new(ListarSalasRequest {});

    // Añadir token JWT al header
    let auth_value = MetadataValue::try_from(format!("Bearer {}", token))
        .map_err(|e| format!("Error al crear header de autorización: {}", e))?;
    request.metadata_mut().insert("authorization", auth_value);

    let response = client
        .listar_salas(request)
        .await
        .map_err(|e| format!("Error gRPC al listar salas: {}", e))?;

    let salas = response
        .into_inner()
        .salas
        .into_iter()
        .map(|s| Sala {
            id: s.id,
            nombre: s.nombre,
            capacidad: s.capacidad,
            activa: s.activa,
        })
        .collect();

    Ok(salas)
}
