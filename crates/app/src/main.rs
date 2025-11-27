use axum::http::Method;
use axum::Router;
use salas_api::routes::salas_routes;
use salas_application::SalaServiceImpl;
use salas_grpc::SalaGrpcServer;
use salas_infrastructure::InMemorySalaRepository;
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::transport::Server;
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

    // Crear routers HTTP
    let api_router = salas_routes(Arc::clone(&service));
    let web_router = app_web::crear_router_web(Arc::clone(&service));

    // Combinar routers HTTP
    let app = Router::new()
        .merge(web_router)
        .nest("/api", api_router)
        .layer(cors);

    // Configurar servidor HTTP
    let http_addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("✓ Servidor HTTP escuchando en http://{}", http_addr);
    tracing::info!("  📱 Web UI:  http://localhost:3000");
    tracing::info!("  🔌 API REST: http://localhost:3000/api/salas");
    tracing::info!("  📚 Swagger:  http://localhost:3000/api/swagger-ui");

    // Configurar servidor gRPC
    let grpc_addr = SocketAddr::from(([0, 0, 0, 0], 50051));
    let grpc_server = SalaGrpcServer::new(Arc::clone(&service));

    // Configurar reflexión para grpcurl
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(salas_grpc::proto::FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();

    tracing::info!("✓ Servidor gRPC escuchando en http://{}", grpc_addr);
    tracing::info!("  🔌 gRPC: http://localhost:50051");

    // Ejecutar ambos servidores en paralelo
    let http_server = async {
        let listener = tokio::net::TcpListener::bind(http_addr).await.unwrap();
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    };

    let grpc_server = async {
        Server::builder()
            .add_service(reflection_service)
            .add_service(grpc_server.into_service())
            .serve(grpc_addr)
            .await
            .unwrap();
    };

    tokio::select! {
        _ = http_server => tracing::error!("Servidor HTTP terminado"),
        _ = grpc_server => tracing::error!("Servidor gRPC terminado"),
    }
}
