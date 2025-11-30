use axum::http::Method;
use axum::Router;
use salas_application::SalaServiceImpl;
use salas_grpc::SalaGrpcServer;
use salas_infrastructure::FileSalaRepository;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tonic::transport::Server;
use tower_http::cors::{Any, CorsLayer};

// Usuarios
use usuarios_application::{AuthService, AuthServiceImpl, UsuarioRepository, UsuarioService, UsuarioServiceImpl};
use usuarios_domain::Rol;
use usuarios_grpc::UsuarioGrpcServer;
use usuarios_infrastructure::FileUsuarioRepository;

#[tokio::main]
async fn main() {
    // Inicializar el sistema de logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    tracing::info!("🚀 Iniciando servidor de Reservas de Salas");

    // ===== SALAS =====
    tracing::info!("📦 Inicializando sistema de Salas...");

    // Crear el repositorio de archivo JSON
    let salas_repository: FileSalaRepository = FileSalaRepository::new(PathBuf::from("./data/salas.json"));

    // Inicializar (cargar datos existentes del archivo)
    salas_repository.init().await
        .expect("Error al inicializar repositorio de salas");

    tracing::info!("✓ Repositorio de salas inicializado (./data/salas.json)");

    // Crear el servicio compartido
    let sala_service: Arc<dyn salas_application::SalaService + Send + Sync> =
        Arc::new(SalaServiceImpl::new(salas_repository));

    tracing::info!("✓ Servicio de salas inicializado");

    // ===== USUARIOS =====
    tracing::info!("👥 Inicializando sistema de Usuarios...");

    // Crear repositorio de usuarios
    let usuarios_repository = FileUsuarioRepository::new(PathBuf::from("./data/usuarios.json"));

    // Inicializar (cargar datos existentes)
    usuarios_repository.init().await
        .expect("Error al inicializar repositorio de usuarios");

    tracing::info!("✓ Repositorio de usuarios inicializado (./data/usuarios.json)");

    // Crear servicios de usuarios
    let usuarios_repo_arc = Arc::new(usuarios_repository);
    let auth_service: Arc<dyn AuthService + Send + Sync> = Arc::new(AuthServiceImpl::new(usuarios_repo_arc.clone()));
    let usuario_service: Arc<dyn UsuarioService + Send + Sync> = Arc::new(UsuarioServiceImpl::new(usuarios_repo_arc.clone()));

    // Crear usuario admin inicial si no existen usuarios
    if usuarios_repo_arc.listar().await.unwrap().is_empty() {
        tracing::info!("🔧 Creando usuario admin inicial...");

        match auth_service.register(
            "Administrador".to_string(),
            "admin@reservas.com".to_string(),
            "admin123".to_string(),
            Some(Rol::Admin),
        ).await {
            Ok(admin_response) => {
                tracing::info!("✅ Usuario admin creado exitosamente:");
                tracing::info!("   📧 Email: {}", admin_response.usuario.email);
                tracing::info!("   👤 Nombre: {}", admin_response.usuario.nombre);
                tracing::info!("   🎫 Token: {}", admin_response.token);
                tracing::warn!("⚠️  IMPORTANTE: Cambia la contraseña del admin ('admin123') en producción");
            }
            Err(e) => {
                tracing::error!("❌ Error al crear admin: {:?}", e);
            }
        }
    } else {
        let usuarios_count = usuarios_repo_arc.listar().await.unwrap().len();
        tracing::info!("✓ Sistema con {} usuario(s) registrado(s)", usuarios_count);
    }

    tracing::info!("✓ Servicios de usuarios inicializados");

    // Configurar CORS para la API REST
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any);

    // Crear routers HTTP
    // Opción 1: Rutas SIN autenticación (para desarrollo/testing)
    // let api_router = salas_api::routes::salas_routes(Arc::clone(&sala_service));
    
    // Opción 2: Rutas CON autenticación (para producción)
    let api_router = salas_api::routes::salas_routes_with_auth(Arc::clone(&sala_service));
    
    let web_router = app_web::crear_router_web(Arc::clone(&sala_service));

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

    // Configurar servidores gRPC
    let grpc_addr = SocketAddr::from(([0, 0, 0, 0], 50051));
    let sala_grpc_server = SalaGrpcServer::new(Arc::clone(&sala_service));
    let usuario_grpc_server = UsuarioGrpcServer::new(Arc::clone(&auth_service), Arc::clone(&usuario_service));

    // Configurar reflexión para grpcurl (incluye ambos servicios)
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(salas_grpc::proto::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(usuarios_grpc::proto::FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();

    tracing::info!("✓ Servidor gRPC escuchando en http://{}", grpc_addr);
    tracing::info!("  🔌 gRPC Salas: http://localhost:50051");
    tracing::info!("  🔌 gRPC Usuarios: http://localhost:50051");

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
            .add_service(sala_grpc_server.into_service())
            .add_service(usuario_grpc_server.into_service())
            .serve(grpc_addr)
            .await
            .unwrap();
    };

    tokio::select! {
        _ = http_server => tracing::error!("Servidor HTTP terminado"),
        _ = grpc_server => tracing::error!("Servidor gRPC terminado"),
    }
}
