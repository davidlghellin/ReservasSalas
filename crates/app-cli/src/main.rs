use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use tonic::{metadata::MetadataValue, Request};

use salas_grpc::proto::{
    sala_service_client::SalaServiceClient, ActivarSalaRequest, CrearSalaRequest,
    DesactivarSalaRequest, ListarSalasRequest, ObtenerSalaRequest,
};
use usuarios_grpc::proto::{usuario_service_client::UsuarioServiceClient, LoginRequest};

const GRPC_URL: &str = "http://localhost:50051";

#[derive(Parser)]
#[command(name = "reservas-cli")]
#[command(about = "Cliente CLI para el Sistema de Reservas con gRPC", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Login para obtener token JWT
    Login {
        #[arg(short, long)]
        email: String,
        #[arg(short, long)]
        password: String,
    },
    /// Gestión de salas (requiere token)
    Sala {
        #[arg(short, long)]
        token: String,
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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Login { email, password } => {
            handle_login(email, password).await?;
        }
        Commands::Sala { token, action } => {
            handle_sala_action(token, action).await?;
        }
    }

    Ok(())
}

async fn handle_login(email: String, password: String) -> Result<()> {
    println!("{}", "🔐 Iniciando sesión...".cyan());

    let mut client = UsuarioServiceClient::connect(GRPC_URL)
        .await
        .context("Error al conectar con el servidor gRPC")?;

    let request = Request::new(LoginRequest { email, password });

    let response = client
        .login(request)
        .await
        .context("Error al hacer login")?;

    let login_response = response.into_inner();
    let usuario = login_response.usuario.expect("Usuario no encontrado en la respuesta");

    println!("{}", "✅ Login exitoso".green().bold());
    println!("  {}: {}", "Usuario".bold(), usuario.id);
    println!("  {}: {}", "Nombre".bold(), usuario.nombre);
    println!("  {}: {}", "Email".bold(), usuario.email);
    println!("  {}: {}", "Rol".bold(), usuario.rol);
    println!("\n{}", "🔑 Token JWT:".yellow().bold());
    println!("{}", login_response.token);
    println!(
        "\n{}",
        "💡 Guarda este token para usarlo en los comandos de sala:".dimmed()
    );
    println!("  {}", format!("--token \"{}\"", login_response.token).dimmed());

    Ok(())
}

async fn handle_sala_action(token: String, action: SalaAction) -> Result<()> {
    let mut client = SalaServiceClient::connect(GRPC_URL)
        .await
        .context("Error al conectar con el servidor gRPC")?;

    match action {
        SalaAction::Crear { nombre, capacidad } => {
            let mut request = Request::new(CrearSalaRequest { nombre, capacidad });
            add_auth_token(&mut request, &token)?;

            match client.crear_sala(request).await {
                Ok(response) => {
                    let sala = response.into_inner();
                    println!("{}", "✅ Sala creada exitosamente".green().bold());
                    print_sala(&sala.id, &sala.nombre, sala.capacidad, sala.activa);
                }
                Err(e) => {
                    println!("{}", format!("❌ Error: {}", e).red());
                }
            }
        }
        SalaAction::Listar => {
            let mut request = Request::new(ListarSalasRequest {});
            add_auth_token(&mut request, &token)?;

            match client.listar_salas(request).await {
                Ok(response) => {
                    let salas = response.into_inner().salas;
                    println!("\n{}", "🏢 Lista de Salas".cyan().bold());
                    println!("{}", "=".repeat(80).cyan());

                    if salas.is_empty() {
                        println!("{}", "No hay salas registradas".yellow());
                    } else {
                        for sala in &salas {
                            print_sala(&sala.id, &sala.nombre, sala.capacidad, sala.activa);
                            println!("{}", "-".repeat(80).dimmed());
                        }
                        println!("{}", format!("Total: {} salas", salas.len()).cyan());
                    }
                }
                Err(e) => {
                    println!("{}", format!("❌ Error: {}", e).red());
                }
            }
        }
        SalaAction::Obtener { id } => {
            let mut request = Request::new(ObtenerSalaRequest { id });
            add_auth_token(&mut request, &token)?;

            match client.obtener_sala(request).await {
                Ok(response) => {
                    let sala = response.into_inner();
                    println!("\n{}", "🏢 Sala".cyan().bold());
                    print_sala(&sala.id, &sala.nombre, sala.capacidad, sala.activa);
                }
                Err(e) => {
                    println!("{}", format!("❌ Error: {}", e).red());
                }
            }
        }
        SalaAction::Activar { id } => {
            let mut request = Request::new(ActivarSalaRequest { id });
            add_auth_token(&mut request, &token)?;

            match client.activar_sala(request).await {
                Ok(_) => {
                    println!("{}", "✅ Sala activada".green().bold());
                }
                Err(e) => {
                    println!("{}", format!("❌ Error: {}", e).red());
                }
            }
        }
        SalaAction::Desactivar { id } => {
            let mut request = Request::new(DesactivarSalaRequest { id });
            add_auth_token(&mut request, &token)?;

            match client.desactivar_sala(request).await {
                Ok(_) => {
                    println!("{}", "✅ Sala desactivada".green().bold());
                }
                Err(e) => {
                    println!("{}", format!("❌ Error: {}", e).red());
                }
            }
        }
    }

    Ok(())
}

fn add_auth_token<T>(request: &mut Request<T>, token: &str) -> Result<()> {
    let auth_value = MetadataValue::try_from(format!("Bearer {}", token))
        .context("Error al crear header de autorización")?;
    request.metadata_mut().insert("authorization", auth_value);
    Ok(())
}

fn print_sala(id: &str, nombre: &str, capacidad: u32, activa: bool) {
    println!("  {}: {}", "ID".bold(), id.dimmed());
    println!("  {}: {}", "Nombre".bold(), nombre);
    println!("  {}: {}", "Capacidad".bold(), capacidad);
    let estado = if activa {
        "Activa ✓".green()
    } else {
        "Inactiva ✗".red()
    };
    println!("  {}: {}", "Estado".bold(), estado);
}
