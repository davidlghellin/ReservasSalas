use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use serde::{Deserialize, Serialize};

const API_BASE_URL: &str = "http://localhost:3000/api";

#[derive(Parser)]
#[command(name = "reservas-cli")]
#[command(about = "Cliente CLI para el Sistema de Reservas (Nueva Arquitectura)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// Gestión de salas
    Sala {
        #[command(subcommand)]
        action: SalaAction,
    },
}
#[derive(Subcommand)]
enum SalaAction {
    /// Crear una nueva sala
    Crear {
        #[arg(short, long)]
        nombre: String,
        #[arg(short, long)]
        capacidad: u32,
    },
    /// Listar todas las salas
    Listar,
    /// Obtener una sala por ID
    Obtener {
        #[arg(short, long)]
        id: String,
    },
    /// Activar una sala
    Activar {
        #[arg(short, long)]
        id: String,
    },
    /// Desactivar una sala
    Desactivar {
        #[arg(short, long)]
        id: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct SalaResponse {
    id: String,
    nombre: String,
    capacidad: u32,
    activa: bool,
}

#[derive(Debug, Serialize)]
struct CrearSalaRequest {
    nombre: String,
    capacidad: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sala { action } => handle_sala_action(action).await?,
    }

    Ok(())
}
async fn handle_sala_action(action: SalaAction) -> Result<()> {
    let client = reqwest::Client::new();

    match action {
        SalaAction::Crear { nombre, capacidad } => {
            let request = CrearSalaRequest { nombre, capacidad };
            let response = client
                .post(format!("{API_BASE_URL}/salas"))
                .json(&request)
                .send()
                .await
                .context("Error al crear sala")?;

            if response.status().is_success() {
                let sala: SalaResponse = response.json().await?;
                println!("{}", "✅ Sala creada exitosamente".green().bold());
                print_sala(&sala);
            } else {
                let error_text = response.text().await?;
                println!("{}", format!("❌ Error: {}", error_text).red());
            }
        }
        SalaAction::Listar => {
            let response = client
                .get(format!("{API_BASE_URL}/salas"))
                .send()
                .await
                .context("Error al listar salas")?;

            if response.status().is_success() {
                let salas: Vec<SalaResponse> = response.json().await?;
                println!("\n{}", "🏢 Lista de Salas".cyan().bold());
                println!("{}", "=".repeat(80).cyan());

                if salas.is_empty() {
                    println!("{}", "No hay salas registradas".yellow());
                } else {
                    for sala in &salas {
                        print_sala(sala);
                        println!("{}", "-".repeat(80).dimmed());
                    }
                    println!("{}", format!("Total: {} salas", salas.len()).cyan());
                }
            } else {
                let error_text = response.text().await?;
                println!("{}", format!("❌ Error: {}", error_text).red());
            }
        }
        SalaAction::Obtener { id } => {
            let response = client
                .get(format!("{API_BASE_URL}/salas/{id}"))
                .send()
                .await
                .context("Error al obtener sala")?;

            if response.status().is_success() {
                let sala: SalaResponse = response.json().await?;
                println!("\n{}", "🏢 Sala".cyan().bold());
                print_sala(&sala);
            } else {
                println!("{}", "❌ Sala no encontrada".red());
            }
        }
        SalaAction::Activar { id } => {
            let response = client
                .post(format!("{API_BASE_URL}/salas/{id}/activar"))
                .send()
                .await
                .context("Error al activar sala")?;

            if response.status().is_success() {
                println!("{}", "✅ Sala activada".green().bold());
            } else {
                println!("{}", "❌ Error al activar sala".red());
            }
        }
        SalaAction::Desactivar { id } => {
            let response = client
                .post(format!("{API_BASE_URL}/salas/{id}/desactivar"))
                .send()
                .await
                .context("Error al desactivar sala")?;

            if response.status().is_success() {
                println!("{}", "✅ Sala desactivada".green().bold());
            } else {
                println!("{}", "❌ Error al desactivar sala".red());
            }
        }
    }

    Ok(())
}
fn print_sala(sala: &SalaResponse) {
    println!("  {}: {}", "ID".bold(), sala.id.dimmed());
    println!("  {}: {}", "Nombre".bold(), sala.nombre);
    println!("  {}: {}", "Capacidad".bold(), sala.capacidad);
    let estado = if sala.activa {
        "Activa ✓".green()
    } else {
        "Inactiva ✗".red()
    };
    println!("  {}: {}", "Estado".bold(), estado);
}
