mod routes;

use axum::http::Method;
use std::net::SocketAddr;
use tower_http::cors::{CorsLayer, Any};

use salas_application::SalaServiceImpl;
use salas_infrastructure::InMemorySalaRepository;

#[tokio::main]
async fn main() {
    // Inicializar el sistema de logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    tracing::info!("Iniciando servidor de Reservas de Salas");

    // Crear el repositorio y el servicio
    let repository = InMemorySalaRepository::new();
    let service = SalaServiceImpl::new(repository);
    tracing::info!("Repositorio y servicio inicializados");

    // Configurar CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any);

    // Crear el router con las rutas
    let app = routes::salas_routes(service).layer(cors);

    // Iniciar el servidor
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Servidor escuchando en http://{}", addr);
    tracing::info!("Endpoints: POST/GET /salas, GET/PUT /salas/{{id}}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
