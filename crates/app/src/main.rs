use axum::http::Method;
use axum::Router;
use salas_api::handlers::SharedSalaService;
use salas_api::routes::salas_routes;
use salas_application::SalaServiceImpl;
use salas_infrastructure::InMemorySalaRepository;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    // Inicializar el sistema de logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    tracing::info!("Iniciando servidor de Reservas de Salas");

    // Crear el repositorio y el servicio
    let repository: InMemorySalaRepository = InMemorySalaRepository::new();
    let service: SalaServiceImpl<InMemorySalaRepository> = SalaServiceImpl::new(repository);

    // 🔹 Envolver en Arc<dyn SalaService + Send + Sync>
    let shared_service: SharedSalaService = Arc::new(service);

    tracing::info!("Repositorio y servicio inicializados");

    // Configurar CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any);

    // Crear el router con las rutas
    // salas_routes ahora recibe SharedSalaService
    let app = Router::new()
        .nest("/salas", salas_routes(shared_service))
        .layer(cors);

    // Iniciar el servidor
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Servidor escuchando en http://{}", addr);
    tracing::info!("Endpoints: POST/GET /salas, GET/PUT /salas/{{id}}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
