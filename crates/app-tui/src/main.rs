use crossterm::{event, terminal, ExecutableCommand};
use ratatui::{prelude::*, widgets::*};
use reqwest::Client;
use serde::Deserialize;

const API_BASE_URL: &str = "http://localhost:3000/api";

#[derive(Deserialize)]
struct Sala {
    id: String,
    nombre: String,
    capacidad: u32,
    activa: bool,
}

enum AppState {
    Menu,
    ListarSalas(Vec<Sala>),
    Error(String),
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    terminal::enable_raw_mode()?;
    std::io::stdout().execute(terminal::EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    let mut state = AppState::Menu;

    loop {
        terminal.draw(|f| match &state {
            AppState::Menu => {
                let block = Block::default().title("Menú").borders(Borders::ALL);
                let items = vec!["1. Sala", "q. Salir"];
                let paragraph = Paragraph::new(items.join("\n"))
                    .block(block)
                    .alignment(Alignment::Left);
                f.render_widget(paragraph, f.size());
            }
            AppState::ListarSalas(salas) => {
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
                f.render_widget(table, f.size());
            }
            AppState::Error(msg) => {
                let paragraph = Paragraph::new(msg.clone())
                    .block(Block::default().title("Error").borders(Borders::ALL))
                    .alignment(Alignment::Center);
                f.render_widget(paragraph, f.size());
            }
        });

        if event::poll(std::time::Duration::from_millis(100))? {
            if let event::Event::Key(key) = event::read()? {
                match &state {
                    AppState::Menu => {
                        if key.code == event::KeyCode::Char('1') {
                            let client = Client::new();
                            let url = format!("{}/salas", API_BASE_URL);
                            match client.get(url).send().await {
                                Ok(resp) => match resp.json::<Vec<Sala>>().await {
                                    Ok(salas) => state = AppState::ListarSalas(salas),
                                    Err(e) => {
                                        state =
                                            AppState::Error(format!("Error parseando JSON: {e}"))
                                    }
                                },
                                Err(e) => {
                                    state =
                                        AppState::Error(format!("Error conectando a la API: {e}"))
                                }
                            }
                        }
                        if key.code == event::KeyCode::Char('q') {
                            break;
                        }
                    }
                    AppState::ListarSalas(_) => {
                        if key.code == event::KeyCode::Char('q') || key.code == event::KeyCode::Esc
                        {
                            state = AppState::Menu;
                        }
                    }
                    AppState::Error(_) => {
                        if key.code == event::KeyCode::Char('q') {
                            break;
                        }
                    }
                }
            }
        }
    }

    terminal::disable_raw_mode()?;
    std::io::stdout().execute(terminal::LeaveAlternateScreen)?;
    Ok(())
}
