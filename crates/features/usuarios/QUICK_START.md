# Quick Start - Sistema de Usuarios

Guía rápida para empezar a usar el sistema de usuarios y autenticación.

## 🚀 Instalación

El sistema ya está instalado en el workspace. Solo necesitas añadirlo a tus dependencias.

## 📝 Uso Básico

### 1. Setup Inicial del Backend

```rust
// En tu main.rs del backend
use std::sync::Arc;
use std::path::PathBuf;
use usuarios_infrastructure::FileUsuarioRepository;
use usuarios_application::{AuthServiceImpl, UsuarioServiceImpl};
use usuarios_domain::Rol;

#[tokio::main]
async fn main() {
    // Crear repositorio
    let repo = FileUsuarioRepository::new(PathBuf::from("./data/usuarios.json"));
    repo.init().await.expect("Error al inicializar repo");

    let repo_arc = Arc::new(repo);

    // Crear servicios
    let auth_service = Arc::new(AuthServiceImpl::new(repo_arc.clone()));
    let usuario_service = Arc::new(UsuarioServiceImpl::new(repo_arc));

    // Crear admin inicial si no existe
    if let Ok(usuarios) = auth_service.listar_usuarios().await {
        if usuarios.is_empty() {
            let admin = auth_service.register(
                "Admin".to_string(),
                "admin@reservas.com".to_string(),
                "admin123".to_string(),
                Some(Rol::Admin),
            ).await.expect("Error al crear admin");

            println!("✅ Admin creado:");
            println!("   Email: {}", admin.usuario.email);
            println!("   Token: {}", admin.token);
        }
    }

    // Usar auth_service y usuario_service en tu app
    // (gRPC, REST, etc.)
}
```

### 2. Registro de Usuario

```rust
use usuarios_application::AuthService;

let response = auth_service.register(
    "Juan Pérez".to_string(),
    "juan@example.com".to_string(),
    "password123".to_string(),
    None, // None = rol Usuario por defecto
).await?;

println!("Usuario registrado:");
println!("  ID: {}", response.usuario.id);
println!("  Nombre: {}", response.usuario.nombre);
println!("  Email: {}", response.usuario.email);
println!("  Token: {}", response.token);
```

### 3. Login

```rust
let login_response = auth_service.login(
    "juan@example.com".to_string(),
    "password123".to_string(),
).await?;

// Guardar el token para próximas requests
let token = login_response.token;
let usuario = login_response.usuario;

println!("Login exitoso:");
println!("  Token: {}", token);
println!("  Usuario: {}", usuario.nombre);
```

### 4. Validar Token

```rust
// En cada request autenticado
let usuario_publico = auth_service
    .validate_token(token)
    .await?;

println!("Token válido para: {}", usuario_publico.nombre);
```

### 5. Operaciones de Usuarios

```rust
use usuarios_application::UsuarioService;

// Listar usuarios
let usuarios = usuario_service.listar_usuarios().await?;
for u in usuarios {
    println!("- {} ({})", u.nombre, u.email);
}

// Obtener usuario específico
let usuario = usuario_service
    .obtener_usuario(user_id)
    .await?;

// Actualizar nombre (el propio usuario)
let updated = usuario_service
    .actualizar_nombre(user_id, "Nuevo Nombre".to_string())
    .await?;

// Cambiar rol (solo admins)
let updated = usuario_service
    .actualizar_rol(admin_id, user_id, Rol::Admin)
    .await?;

// Desactivar usuario (solo admins)
usuario_service
    .desactivar_usuario(admin_id, user_id)
    .await?;
```

## 🔐 Verificación de Roles

```rust
use usuarios_auth::JwtService;

// Validar token y verificar rol
let claims = JwtService::validate_token(&token)?;

if JwtService::is_admin_token(&claims) {
    println!("✅ Usuario es admin");
    // Permitir operaciones de admin
} else {
    println!("❌ Usuario no es admin");
    // Denegar acceso
}
```

## 🛡️ Middleware de Autenticación (Ejemplo Axum)

```rust
use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Extension,
};
use usuarios_auth::{JwtService, Claims};

async fn auth_middleware<B>(
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // Extraer token del header
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validar token
    let claims = JwtService::validate_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Agregar claims a las extensiones del request
    req.extensions_mut().insert(claims.clone());

    Ok(next.run(req).await)
}

// Usar en rutas
async fn protected_route(
    Extension(claims): Extension<Claims>,
) -> String {
    format!("Hola, {}!", claims.email)
}

// Middleware solo para admins
async fn admin_middleware<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let claims = req.extensions().get::<Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !JwtService::is_admin_token(claims) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}
```

## 📱 Ejemplo Frontend (Iced)

```rust
use iced::{Task, Element};
use usuarios_domain::UsuarioPublico;

#[derive(Debug, Clone)]
enum Message {
    Login,
    LoginSuccess(String, UsuarioPublico),
    LoginError(String),
    Logout,
}

struct App {
    email: String,
    password: String,
    token: Option<String>,
    usuario: Option<UsuarioPublico>,
    error: Option<String>,
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Login => {
                let email = self.email.clone();
                let password = self.password.clone();

                Task::perform(
                    async move {
                        // Llamar a gRPC login
                        let response = grpc_client
                            .login(email, password)
                            .await?;
                        Ok((response.token, response.usuario))
                    },
                    |result| match result {
                        Ok((token, usuario)) => Message::LoginSuccess(token, usuario),
                        Err(e) => Message::LoginError(e.to_string()),
                    }
                )
            }

            Message::LoginSuccess(token, usuario) => {
                self.token = Some(token);
                self.usuario = Some(usuario);
                self.error = None;
                Task::none()
            }

            Message::LoginError(error) => {
                self.error = Some(error);
                Task::none()
            }

            Message::Logout => {
                self.token = None;
                self.usuario = None;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        if let Some(usuario) = &self.usuario {
            // Mostrar pantalla principal
            column![
                text(format!("Bienvenido, {}!", usuario.nombre)),
                button("Cerrar sesión").on_press(Message::Logout),
            ].into()
        } else {
            // Mostrar login
            column![
                text_input("Email", &self.email),
                text_input("Contraseña", &self.password).password(),
                button("Iniciar sesión").on_press(Message::Login),
                if let Some(error) = &self.error {
                    text(error).style(Color::from_rgb(1.0, 0.0, 0.0))
                } else {
                    text("")
                }
            ].into()
        }
    }
}
```

## 🧪 Tests Rápidos

```rust
#[tokio::test]
async fn test_flujo_completo() {
    use usuarios_infrastructure::FileUsuarioRepository;
    use usuarios_application::AuthServiceImpl;
    use std::sync::Arc;
    use tempfile::TempDir;

    // Setup
    let temp_dir = TempDir::new().unwrap();
    let repo = FileUsuarioRepository::new(
        temp_dir.path().join("usuarios.json")
    );
    repo.init().await.unwrap();

    let auth_service = AuthServiceImpl::new(Arc::new(repo));

    // Registrar
    let register = auth_service.register(
        "Test User".to_string(),
        "test@test.com".to_string(),
        "password123".to_string(),
        None,
    ).await.unwrap();

    assert!(!register.token.is_empty());

    // Login
    let login = auth_service.login(
        "test@test.com".to_string(),
        "password123".to_string(),
    ).await.unwrap();

    assert!(!login.token.is_empty());

    // Validar token
    let usuario = auth_service
        .validate_token(login.token)
        .await.unwrap();

    assert_eq!(usuario.email, "test@test.com");
}
```

## 📋 Checklist de Implementación

### Backend
- [ ] Crear repositorio (FileUsuarioRepository)
- [ ] Inicializar con `.init().await`
- [ ] Crear AuthService y UsuarioService
- [ ] Crear usuario admin inicial
- [ ] Implementar endpoints gRPC/REST
- [ ] Agregar middleware de autenticación
- [ ] Agregar middleware de autorización (admin)

### Frontend
- [ ] Crear pantalla de login
- [ ] Guardar token en estado
- [ ] Incluir token en headers de requests
- [ ] Manejar errores de autenticación
- [ ] Implementar logout
- [ ] Mostrar información del usuario logueado

### Seguridad
- [ ] Cambiar JWT_SECRET a variable de entorno
- [ ] Configurar HTTPS en producción
- [ ] Implementar rate limiting en login
- [ ] Agregar refresh tokens (opcional)
- [ ] Implementar "recordarme" (opcional)

## 🔧 Troubleshooting

### Error: "Email ya está registrado"
```rust
// Verificar si el email existe antes de registrar
if auth_service.existe_email(&email).await? {
    println!("Email ya registrado");
}
```

### Error: "Credenciales inválidas"
- Verifica que el email sea correcto
- Verifica que la contraseña coincida
- Verifica que el usuario esté activo

### Error: "Token inválido"
- El token puede haber expirado (24 horas)
- El token puede estar malformado
- La clave secreta puede haber cambiado

### Error: "Permisos denegados"
- Verifica que el usuario tenga rol Admin
- Verifica que el token sea válido

## 📚 Siguiente: Integración con gRPC

Ver [FASE_0_USUARIOS_AUTH.md](../../../FASE_0_USUARIOS_AUTH.md) para:
- Definir proto files
- Crear servidor gRPC
- Implementar middleware de auth
- Conectar desde Iced

---

**¡El sistema está listo para usar! 🎉**
