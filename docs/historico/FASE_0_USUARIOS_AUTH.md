# 🔐 Fase 0: Usuarios + Autenticación JWT

**Prioridad:** ALTA - Hacer esto ANTES de Reservas
**Tiempo estimado:** 1 semana
**Motivo:** Las reservas necesitan saber quién las crea, y necesitas controlar acceso

---

## 🎯 Objetivos

1. ✅ Entidad Usuario (dominio)
2. ✅ Sistema de autenticación con JWT
3. ✅ Registro y login
4. ✅ Roles (Admin, Usuario)
5. ✅ Middleware de autorización
6. ✅ Integración con gRPC y REST

---

## 📦 Arquitectura de Usuarios

```
crates/features/usuarios/
├── domain/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── usuario.rs          # Entidad Usuario
│   │   ├── rol.rs              # Enum Rol
│   │   └── error.rs            # UsuarioError
│   └── Cargo.toml
├── application/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── ports.rs            # UsuarioRepository trait
│   │   ├── service.rs          # UsuarioService
│   │   └── auth_service.rs     # AuthService (login, JWT)
│   └── Cargo.toml
├── infrastructure/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── memory_repository.rs
│   │   └── file_repository.rs
│   └── Cargo.toml
├── grpc/
│   ├── proto/
│   │   └── usuario.proto
│   ├── src/
│   │   ├── lib.rs
│   │   └── server.rs
│   └── Cargo.toml
└── auth/                        # ← NUEVO: crate de autenticación
    ├── src/
    │   ├── lib.rs
    │   ├── jwt.rs               # Generar/validar tokens
    │   ├── password.rs          # Hash de contraseñas
    │   └── middleware.rs        # Middleware gRPC/REST
    └── Cargo.toml
```

---

## 🏗️ Paso 1: Domain - Entidad Usuario

### 1.1 Crear estructura

```bash
mkdir -p crates/features/usuarios/{domain,application,infrastructure,grpc,auth}/src
```

### 1.2 Usuario (entidad de dominio)

```rust
// crates/features/usuarios/domain/src/usuario.rs
use serde::{Deserialize, Serialize};
use validator::Validate;
use crate::error::UsuarioError;
use crate::rol::Rol;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct Usuario {
    pub id: String,

    #[validate(email(message = "Email inválido"))]
    pub email: String,

    #[validate(length(min = 2, max = 100))]
    pub nombre: String,

    /// Hash de la contraseña (NUNCA la contraseña en texto plano)
    pub password_hash: String,

    pub rol: Rol,

    pub activo: bool,

    /// Timestamp de creación
    pub created_at: i64,
}

impl Usuario {
    pub fn new(
        id: String,
        email: String,
        nombre: String,
        password_hash: String,
        rol: Rol,
    ) -> Result<Self, UsuarioError> {
        let usuario = Self {
            id,
            email: email.trim().to_lowercase(),
            nombre: nombre.trim().to_string(),
            password_hash,
            rol,
            activo: true,
            created_at: chrono::Utc::now().timestamp(),
        };

        // Validar
        usuario.validate()
            .map_err(|e| UsuarioError::Validacion(format!("{}", e)))?;

        Ok(usuario)
    }

    pub fn es_admin(&self) -> bool {
        self.rol == Rol::Admin
    }

    pub fn desactivar(&mut self) {
        self.activo = false;
    }

    pub fn activar(&mut self) {
        self.activo = true;
    }
}
```

### 1.3 Rol (enum)

```rust
// crates/features/usuarios/domain/src/rol.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rol {
    Admin,      // Puede gestionar salas y ver todas las reservas
    Usuario,    // Solo puede crear sus propias reservas
}

impl Default for Rol {
    fn default() -> Self {
        Rol::Usuario
    }
}

impl std::fmt::Display for Rol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rol::Admin => write!(f, "Admin"),
            Rol::Usuario => write!(f, "Usuario"),
        }
    }
}
```

### 1.4 Error

```rust
// crates/features/usuarios/domain/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UsuarioError {
    #[error("Validación: {0}")]
    Validacion(String),

    #[error("Usuario no encontrado")]
    NoEncontrado,

    #[error("Email ya registrado")]
    EmailDuplicado,

    #[error("Credenciales inválidas")]
    CredencialesInvalidas,

    #[error("Usuario inactivo")]
    UsuarioInactivo,

    #[error("Error de repositorio: {0}")]
    ErrorRepositorio(String),

    #[error("No autorizado")]
    NoAutorizado,
}
```

### 1.5 Cargo.toml

```toml
# crates/features/usuarios/domain/Cargo.toml
[package]
name = "usuarios-domain"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true, features = ["derive"] }
validator = { version = "0.16", features = ["derive"] }
thiserror = "1.0"
chrono = "0.4"
```

---

## 🏗️ Paso 2: Auth - JWT y Passwords

### 2.1 Password hashing

```rust
// crates/features/usuarios/auth/src/password.rs
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub struct PasswordService;

impl PasswordService {
    /// Hash de contraseña (almacenar en DB)
    pub fn hash_password(password: &str) -> Result<String, String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| format!("Error al hashear: {}", e))?
            .to_string();

        Ok(hash)
    }

    /// Verificar contraseña
    pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| format!("Hash inválido: {}", e))?;

        let argon2 = Argon2::default();

        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
}
```

### 2.2 JWT (JSON Web Tokens)

```rust
// crates/features/usuarios/auth/src/jwt.rs
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};
use usuarios_domain::Rol;

const JWT_SECRET: &str = "tu-secret-super-seguro"; // ← EN PRODUCCIÓN: env var
const TOKEN_EXPIRATION_HOURS: i64 = 24;

/// Claims del JWT (payload)
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // Subject (user id)
    pub email: String,
    pub rol: String,        // "Admin" o "Usuario"
    pub exp: usize,         // Expiration time
    pub iat: usize,         // Issued at
}

pub struct JwtService;

impl JwtService {
    /// Generar token JWT
    pub fn generate_token(user_id: &str, email: &str, rol: Rol) -> Result<String, String> {
        let now = Utc::now();
        let exp = (now + Duration::hours(TOKEN_EXPIRATION_HOURS)).timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            rol: rol.to_string(),
            exp,
            iat: now.timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(JWT_SECRET.as_ref()),
        )
        .map_err(|e| format!("Error al generar token: {}", e))
    }

    /// Validar token JWT
    pub fn validate_token(token: &str) -> Result<Claims, String> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(JWT_SECRET.as_ref()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| format!("Token inválido: {}", e))
    }

    /// Extraer user_id del token
    pub fn extract_user_id(token: &str) -> Result<String, String> {
        Self::validate_token(token).map(|claims| claims.sub)
    }
}
```

### 2.3 Cargo.toml

```toml
# crates/features/usuarios/auth/Cargo.toml
[package]
name = "usuarios-auth"
version = "0.1.0"
edition = "2021"

[dependencies]
usuarios-domain = { path = "../domain" }

# JWT
jsonwebtoken = "9"

# Password hashing (Argon2 - más seguro que bcrypt)
argon2 = "0.5"
password-hash = "0.5"

# Utils
serde = { workspace = true, features = ["derive"] }
chrono = "0.4"
```

---

## 🏗️ Paso 3: Application - Services

### 3.1 UsuarioRepository trait

```rust
// crates/features/usuarios/application/src/ports.rs
use async_trait::async_trait;
use usuarios_domain::{Usuario, UsuarioError};

#[async_trait]
pub trait UsuarioRepository: Send + Sync {
    async fn guardar(&self, usuario: &Usuario) -> Result<(), UsuarioError>;
    async fn obtener(&self, id: &str) -> Result<Option<Usuario>, UsuarioError>;
    async fn obtener_por_email(&self, email: &str) -> Result<Option<Usuario>, UsuarioError>;
    async fn listar(&self) -> Result<Vec<Usuario>, UsuarioError>;
    async fn actualizar(&self, usuario: &Usuario) -> Result<(), UsuarioError>;
    async fn existe_email(&self, email: &str) -> Result<bool, UsuarioError>;
}
```

### 3.2 AuthService

```rust
// crates/features/usuarios/application/src/auth_service.rs
use async_trait::async_trait;
use usuarios_domain::{Usuario, UsuarioError, Rol};
use usuarios_auth::{PasswordService, JwtService};
use crate::ports::UsuarioRepository;
use std::sync::Arc;

#[async_trait]
pub trait AuthService: Send + Sync {
    async fn registrar(&self, email: String, nombre: String, password: String) -> Result<String, UsuarioError>;
    async fn login(&self, email: String, password: String) -> Result<String, UsuarioError>;
    async fn validar_token(&self, token: &str) -> Result<Usuario, UsuarioError>;
}

pub struct AuthServiceImpl<R: UsuarioRepository> {
    repository: Arc<R>,
}

impl<R: UsuarioRepository> AuthServiceImpl<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: UsuarioRepository> AuthService for AuthServiceImpl<R> {
    /// Registrar nuevo usuario
    async fn registrar(
        &self,
        email: String,
        nombre: String,
        password: String,
    ) -> Result<String, UsuarioError> {
        // Verificar que el email no exista
        if self.repository.existe_email(&email).await? {
            return Err(UsuarioError::EmailDuplicado);
        }

        // Hash de contraseña
        let password_hash = PasswordService::hash_password(&password)
            .map_err(|e| UsuarioError::ErrorRepositorio(e))?;

        // Crear usuario
        let id = uuid::Uuid::new_v4().to_string();
        let usuario = Usuario::new(id.clone(), email, nombre, password_hash, Rol::Usuario)?;

        // Guardar en repositorio
        self.repository.guardar(&usuario).await?;

        // Generar token JWT
        let token = JwtService::generate_token(&usuario.id, &usuario.email, usuario.rol)
            .map_err(|e| UsuarioError::ErrorRepositorio(e))?;

        Ok(token)
    }

    /// Login de usuario
    async fn login(&self, email: String, password: String) -> Result<String, UsuarioError> {
        // Buscar usuario por email
        let usuario = self.repository.obtener_por_email(&email).await?
            .ok_or(UsuarioError::CredencialesInvalidas)?;

        // Verificar que esté activo
        if !usuario.activo {
            return Err(UsuarioError::UsuarioInactivo);
        }

        // Verificar contraseña
        let valid = PasswordService::verify_password(&password, &usuario.password_hash)
            .map_err(|e| UsuarioError::ErrorRepositorio(e))?;

        if !valid {
            return Err(UsuarioError::CredencialesInvalidas);
        }

        // Generar token JWT
        let token = JwtService::generate_token(&usuario.id, &usuario.email, usuario.rol)
            .map_err(|e| UsuarioError::ErrorRepositorio(e))?;

        Ok(token)
    }

    /// Validar token y obtener usuario
    async fn validar_token(&self, token: &str) -> Result<Usuario, UsuarioError> {
        let claims = JwtService::validate_token(token)
            .map_err(|_| UsuarioError::NoAutorizado)?;

        let usuario = self.repository.obtener(&claims.sub).await?
            .ok_or(UsuarioError::NoEncontrado)?;

        if !usuario.activo {
            return Err(UsuarioError::UsuarioInactivo);
        }

        Ok(usuario)
    }
}
```

---

## 🏗️ Paso 4: gRPC con Autenticación

### 4.1 Protocol Buffer

```protobuf
// crates/features/usuarios/grpc/proto/usuario.proto
syntax = "proto3";

package usuario;

service UsuarioService {
  // Auth endpoints (públicos - no requieren token)
  rpc Registrar(RegistrarRequest) returns (AuthResponse);
  rpc Login(LoginRequest) returns (AuthResponse);

  // Usuario endpoints (requieren token)
  rpc ObtenerPerfil(Empty) returns (UsuarioResponse);
  rpc ActualizarPerfil(ActualizarPerfilRequest) returns (UsuarioResponse);

  // Admin endpoints (solo admin)
  rpc ListarUsuarios(Empty) returns (ListarUsuariosResponse);
}

message RegistrarRequest {
  string email = 1;
  string nombre = 2;
  string password = 3;
}

message LoginRequest {
  string email = 1;
  string password = 2;
}

message AuthResponse {
  string token = 1;
  UsuarioResponse usuario = 2;
}

message UsuarioResponse {
  string id = 1;
  string email = 2;
  string nombre = 3;
  string rol = 4;
  bool activo = 5;
}

message ActualizarPerfilRequest {
  string nombre = 1;
}

message ListarUsuariosResponse {
  repeated UsuarioResponse usuarios = 1;
}

message Empty {}
```

### 4.2 Middleware de autenticación

```rust
// crates/features/usuarios/auth/src/middleware.rs
use tonic::{Request, Status};
use usuarios_domain::Usuario;
use crate::jwt::JwtService;

/// Metadata key para el token JWT
pub const AUTH_HEADER: &str = "authorization";

/// Extraer usuario del token en metadata
pub fn extract_user_from_request<T>(request: &Request<T>) -> Result<String, Status> {
    let token = request
        .metadata()
        .get(AUTH_HEADER)
        .ok_or_else(|| Status::unauthenticated("Token no proporcionado"))?
        .to_str()
        .map_err(|_| Status::unauthenticated("Token inválido"))?;

    // Remover "Bearer " si existe
    let token = token.trim_start_matches("Bearer ").trim();

    JwtService::extract_user_id(token)
        .map_err(|_| Status::unauthenticated("Token inválido o expirado"))
}

/// Verificar que el usuario es admin
pub fn require_admin(usuario: &Usuario) -> Result<(), Status> {
    if !usuario.es_admin() {
        return Err(Status::permission_denied("Requiere rol de administrador"));
    }
    Ok(())
}
```

### 4.3 Servidor gRPC con Auth

```rust
// crates/features/usuarios/grpc/src/server.rs
use tonic::{Request, Response, Status};
use usuarios_application::{AuthService, UsuarioService};
use usuarios_auth::middleware::{extract_user_from_request, require_admin};
use std::sync::Arc;

use crate::proto::{
    usuario_service_server::UsuarioService as UsuarioServiceTrait,
    RegistrarRequest, LoginRequest, AuthResponse, UsuarioResponse,
    // ...
};

pub struct UsuarioGrpcServer {
    auth_service: Arc<dyn AuthService + Send + Sync>,
    usuario_service: Arc<dyn UsuarioService + Send + Sync>,
}

#[tonic::async_trait]
impl UsuarioServiceTrait for UsuarioGrpcServer {
    /// Endpoint público
    async fn registrar(
        &self,
        request: Request<RegistrarRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();

        let token = self.auth_service
            .registrar(req.email, req.nombre, req.password)
            .await
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        // Obtener usuario...
        Ok(Response::new(AuthResponse { token, usuario: Some(...) }))
    }

    /// Endpoint público
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();

        let token = self.auth_service
            .login(req.email, req.password)
            .await
            .map_err(|e| Status::unauthenticated(e.to_string()))?;

        Ok(Response::new(AuthResponse { token, usuario: Some(...) }))
    }

    /// Endpoint protegido - requiere token
    async fn obtener_perfil(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<UsuarioResponse>, Status> {
        // Extraer user_id del token JWT
        let user_id = extract_user_from_request(&request)?;

        let usuario = self.usuario_service
            .obtener_usuario(&user_id)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;

        Ok(Response::new(usuario.into()))
    }

    /// Endpoint solo admin
    async fn listar_usuarios(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<ListarUsuariosResponse>, Status> {
        let user_id = extract_user_from_request(&request)?;

        let usuario = self.usuario_service.obtener_usuario(&user_id).await
            .map_err(|e| Status::internal(e.to_string()))?;

        // Verificar que es admin
        require_admin(&usuario)?;

        let usuarios = self.usuario_service.listar_usuarios().await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ListarUsuariosResponse {
            usuarios: usuarios.into_iter().map(|u| u.into()).collect()
        }))
    }
}
```

---

## 🏗️ Paso 5: Cliente Iced con Auth

### 5.1 Login screen

```rust
// En Iced
struct LoginScreen {
    email: String,
    password: String,
    error: String,
}

#[derive(Debug, Clone)]
enum LoginMessage {
    EmailChanged(String),
    PasswordChanged(String),
    Login,
    LoginResult(Result<String, String>), // Token
}

fn update(&mut self, message: LoginMessage) -> Task<Message> {
    match message {
        LoginMessage::Login => {
            let email = self.email.clone();
            let password = self.password.clone();

            Task::perform(
                async move {
                    // Llamar a gRPC
                    login_grpc(email, password).await
                },
                LoginMessage::LoginResult
            )
        }

        LoginMessage::LoginResult(Ok(token)) => {
            // Guardar token (por ejemplo, en un archivo)
            save_token(&token);

            // Cambiar a pantalla principal
            Task::none()
        }

        LoginMessage::LoginResult(Err(e)) => {
            self.error = e;
            Task::none()
        }
        // ...
    }
}
```

### 5.2 Enviar token en requests

```rust
// Al hacer requests gRPC, añadir metadata
async fn listar_salas_con_auth(token: &str) -> Result<Vec<Sala>, String> {
    let mut client = SalaServiceClient::connect("http://localhost:50051").await?;

    let mut request = Request::new(ListarSalasRequest {});

    // ✅ Añadir token JWT en metadata
    request.metadata_mut().insert(
        "authorization",
        format!("Bearer {}", token).parse().unwrap(),
    );

    let response = client.listar_salas(request).await?;
    Ok(response.into_inner().salas)
}
```

---

## 📊 Resumen de cambios

### Nuevo código:
- `crates/features/usuarios/` (completo)
- Middleware de auth en gRPC
- Pantallas de login/registro en Iced

### Modificar:
- `crates/app/src/main.rs` → Inicializar `UsuarioService` y `AuthService`
- Salas gRPC → Añadir verificación de admin para crear/editar salas
- Reservas (futuro) → Guardar `usuario_id` en cada reserva

### Tiempo estimado:
- Domain + Auth crate: 1-2 días
- Application + Infrastructure: 1 día
- gRPC + middleware: 1 día
- Iced login/registro: 1-2 días

**Total: ~1 semana**

---

## 🎯 Próximo paso

¿Quieres que empiece creando la estructura de `usuarios/domain` con la entidad Usuario?
