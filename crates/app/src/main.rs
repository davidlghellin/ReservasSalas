use axum::http::Method;
use axum::Router;
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

    tracing::info!("🚀 Iniciando servidor de Reservas de Salas");

    // Crear el repositorio y el servicio compartido
    let repository = InMemorySalaRepository::new();
    let service: Arc<dyn salas_application::SalaService + Send + Sync> =
        Arc::new(SalaServiceImpl::new(repository));

    tracing::info!("✓ Repositorio y servicio inicializados");

    // Configurar CORS para la API REST
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any);

    // Crear routers
    let api_router = salas_routes(Arc::clone(&service));
    let web_router = app_web::crear_router_web(Arc::clone(&service));

    // Combinar routers
    let app = Router::new()
        .merge(web_router)
        .nest("/api", api_router)
        .layer(cors);

    // Iniciar el servidor
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("✓ Servidor escuchando en http://{}", addr);
    tracing::info!("  📱 Web UI:  http://localhost:3000");
    tracing::info!("  🔌 API REST: http://localhost:3000/api/salas");
    tracing::info!("  📚 Swagger:  http://localhost:3000/api/swagger-ui");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
