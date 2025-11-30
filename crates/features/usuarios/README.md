# Sistema de Usuarios y Autenticación

Sistema completo de gestión de usuarios con autenticación JWT para el proyecto de Reservas de Salas.

## 🏗️ Arquitectura Hexagonal

```
┌─────────────────────────────────────────┐
│           Domain (Core)                  │
│  • Usuario (entity)                      │
│  • Rol (enum: Admin, Usuario)           │
│  • UsuarioError                          │
│  • Reglas de negocio y validaciones      │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│              Auth                        │
│  • JwtService (generar/validar tokens)   │
│  • PasswordService (Argon2 hashing)      │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│        Application (Ports)               │
│  • AuthService (login, register)         │
│  • UsuarioService (CRUD usuarios)        │
│  • UsuarioRepository trait ← PORT        │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│     Infrastructure (Adapters)            │
│  • FileUsuarioRepository (JSON file)     │
│  • PostgresRepository (futuro)           │
└──────────────────────────────────────────┘
```

## 📦 Crates Creados

### 1. `usuarios-domain`

**Ubicación:** `crates/features/usuarios/domain/`

Contiene las entidades y reglas de negocio del dominio de usuarios.

**Entidades:**
- `Usuario` - Entidad principal con validaciones
- `Rol` - Enum: Admin, Usuario
- `UsuarioError` - Errores del dominio

**Características:**
- ✅ Validaciones de nombre (2-100 caracteres)
- ✅ Validaciones de email (formato correcto)
- ✅ Validaciones de contraseña (mínimo 8 caracteres)
- ✅ Control de usuario activo/inactivo
- ✅ Timestamps de creación y actualización
- ✅ 15+ tests unitarios

**Ejemplo:**
```rust
use usuarios_domain::{Usuario, Rol, validar_password};

// Validar contraseña antes de hashear
validar_password("mypassword123")?;

// Crear usuario con hash de contraseña
let usuario = Usuario::new(
    "Juan Pérez".to_string(),
    "juan@example.com".to_string(),
    password_hash,
    Rol::Usuario,
)?;

// Operaciones
usuario.es_admin(); // false
usuario.desactivar();
usuario.sin_password(); // UsuarioPublico sin hash
```

---

### 2. `usuarios-auth`

**Ubicación:** `crates/features/usuarios/auth/`

Servicios de autenticación y seguridad.

**Servicios:**
- `PasswordService` - Hash y verificación con Argon2
- `JwtService` - Generación y validación de tokens JWT

**Características:**
- 🔐 Argon2 para hashing (más seguro que bcrypt)
- 🎫 JWT con expiración de 24 horas
- ✅ Salt aleatorio por contraseña
- ✅ Verificación de roles en tokens
- ✅ 10+ tests de seguridad

**Ejemplo - Password:**
```rust
use usuarios_auth::PasswordService;

// Hashear contraseña
let hash = PasswordService::hash_password("mypassword")?;
// Resultado: "$argon2id$v=19$m=19456,t=2,p=1$..."

// Verificar contraseña
let es_valida = PasswordService::verify_password("mypassword", &hash)?;
// true
```

**Ejemplo - JWT:**
```rust
use usuarios_auth::JwtService;
use usuarios_domain::Rol;

// Generar token
let token = JwtService::generate_token(
    "user-123",
    "user@example.com",
    Rol::Usuario
)?;

// Validar token
let claims = JwtService::validate_token(&token)?;
assert_eq!(claims.sub, "user-123");
assert_eq!(claims.email, "user@example.com");
assert_eq!(claims.rol, "usuario");

// Verificar si es admin
JwtService::is_admin_token(&claims); // false
```

---

### 3. `usuarios-application`

**Ubicación:** `crates/features/usuarios/application/`

Casos de uso y lógica de aplicación.

**Servicios:**

#### `AuthService`
```rust
pub trait AuthService {
    async fn register(...) -> Result<RegisterResponse, UsuarioError>;
    async fn login(...) -> Result<LoginResponse, UsuarioError>;
    async fn validate_token(...) -> Result<UsuarioPublico, UsuarioError>;
    async fn change_password(...) -> Result<(), UsuarioError>;
}
```

#### `UsuarioService`
```rust
pub trait UsuarioService {
    async fn obtener_usuario(...) -> Result<UsuarioPublico, UsuarioError>;
    async fn listar_usuarios(...) -> Result<Vec<UsuarioPublico>, UsuarioError>;
    async fn actualizar_nombre(...) -> Result<UsuarioPublico, UsuarioError>;
    async fn actualizar_rol(...) -> Result<UsuarioPublico, UsuarioError>;
    async fn desactivar_usuario(...) -> Result<(), UsuarioError>;
    async fn activar_usuario(...) -> Result<(), UsuarioError>;
}
```

**Características:**
- ✅ Verifica email no duplicado al registrar
- ✅ Hashea contraseñas automáticamente
- ✅ Solo usuarios activos pueden hacer login
- ✅ Solo admins pueden cambiar roles
- ✅ Admins no pueden desactivarse a sí mismos
- ✅ 15+ tests de casos de uso

**Ejemplo - Registro:**
```rust
use usuarios_application::{AuthServiceImpl, RegisterResponse};
use usuarios_domain::Rol;

let auth_service = AuthServiceImpl::new(repository);

// Registrar usuario
let response = auth_service.register(
    "Juan Pérez".to_string(),
    "juan@example.com".to_string(),
    "password123".to_string(),
    Some(Rol::Usuario),
).await?;

println!("Token: {}", response.token);
println!("Usuario: {:?}", response.usuario);
```

**Ejemplo - Login:**
```rust
let login_response = auth_service.login(
    "juan@example.com".to_string(),
    "password123".to_string(),
).await?;

// Usar el token en requests
let auth_header = format!("Bearer {}", login_response.token);
```

---

### 4. `usuarios-infrastructure`

**Ubicación:** `crates/features/usuarios/infrastructure/`

Adaptadores de persistencia.

**Adaptadores:**
- `FileUsuarioRepository` - Persistencia en JSON

**Características:**
- 💾 Persistencia en archivo JSON
- ⚡ Cache en memoria con `RwLock`
- 🔒 Thread-safe para concurrencia
- 📁 Auto-crea directorios
- ✅ 10+ tests de persistencia

**Formato JSON:**
```json
{
  "usuarios": {
    "user-uuid-123": {
      "id": "user-uuid-123",
      "nombre": "Juan Pérez",
      "email": "juan@example.com",
      "password_hash": "$argon2id$v=19$...",
      "rol": "Usuario",
      "created_at": "2024-11-30T10:00:00Z",
      "updated_at": "2024-11-30T10:00:00Z",
      "activo": true
    }
  }
}
```

**Uso:**
```rust
use usuarios_infrastructure::FileUsuarioRepository;
use usuarios_application::UsuarioRepository;
use std::path::PathBuf;

// Crear repositorio
let repo = FileUsuarioRepository::new(
    PathBuf::from("./data/usuarios.json")
);

// O usar ruta por defecto
let repo = FileUsuarioRepository::default_path();

// ⚠️ IMPORTANTE: Inicializar para cargar datos existentes
repo.init().await?;

// Usar repositorio
repo.guardar(&usuario).await?;
let usuario = repo.obtener_por_email("juan@example.com").await?;
```

---

## 🚀 Ejemplo Completo: Setup Backend

```rust
use std::sync::Arc;
use usuarios_infrastructure::FileUsuarioRepository;
use usuarios_application::{AuthServiceImpl, UsuarioServiceImpl};
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    // 1. Crear repositorio
    let repo = FileUsuarioRepository::new(
        PathBuf::from("./data/usuarios.json")
    );

    // 2. Inicializar (cargar datos existentes)
    repo.init().await.expect("Error al inicializar repositorio");

    // 3. Crear servicios compartidos
    let repo_arc = Arc::new(repo);
    let auth_service = Arc::new(AuthServiceImpl::new(repo_arc.clone()));
    let usuario_service = Arc::new(UsuarioServiceImpl::new(repo_arc.clone()));

    // 4. Registrar primer admin (si no existe)
    if repo_arc.listar().await.unwrap().is_empty() {
        println!("🔧 Creando usuario admin inicial...");

        let admin_response = auth_service.register(
            "Admin".to_string(),
            "admin@reservas.com".to_string(),
            "admin123".to_string(),
            Some(Rol::Admin),
        ).await.expect("Error al crear admin");

        println!("✅ Admin creado: {}", admin_response.usuario.email);
        println!("🎫 Token: {}", admin_response.token);
    }

    // 5. Usar servicios en tu aplicación
    // (gRPC, REST, etc.)
}
```

---

## 🔐 Flujo de Autenticación

### 1. Registro
```
Usuario → Frontend → Backend
                       ↓
              AuthService.register()
                       ↓
              validar_password()
              verificar_email_no_existe()
              hashear_contraseña_argon2()
              crear_usuario()
              guardar_en_repositorio()
              generar_token_jwt()
                       ↓
              { token, usuario } → Frontend
```

### 2. Login
```
Usuario → Frontend → Backend
                       ↓
              AuthService.login()
                       ↓
              buscar_por_email()
              verificar_usuario_activo()
              verificar_password_argon2()
              generar_token_jwt()
                       ↓
              { token, usuario } → Frontend
```

### 3. Request Autenticado
```
Frontend → Header: "Authorization: Bearer <token>"
              ↓
         Backend (middleware)
              ↓
         JwtService.validate_token()
              ↓
         verificar_usuario_activo()
              ↓
         Ejecutar operación
```

---

## 🧪 Tests

Cada crate incluye tests completos:

```bash
# Tests de dominio (validaciones, entidades)
cargo test --package usuarios-domain

# Tests de auth (JWT, Argon2)
cargo test --package usuarios-auth

# Tests de application (casos de uso)
cargo test --package usuarios-application

# Tests de infrastructure (persistencia)
cargo test --package usuarios-infrastructure

# Todos los tests
cargo test --workspace
```

**Cobertura de tests:**
- Domain: 15+ tests
- Auth: 10+ tests
- Application: 15+ tests
- Infrastructure: 10+ tests
- **Total: 50+ tests**

---

## 📊 Comparación con Salas

| Característica | Salas | Usuarios |
|----------------|-------|----------|
| **Domain** | Sala, SalaError | Usuario, Rol, UsuarioError |
| **Application** | SalaService | AuthService + UsuarioService |
| **Infrastructure** | InMemory + File | File (JSON) |
| **Auth** | N/A | JWT + Argon2 |
| **Validaciones** | Nombre, Capacidad | Nombre, Email, Password |
| **Tests** | 20+ | 50+ |

---

## 🔜 Próximos Pasos

### 1. gRPC Server (Fase pendiente)
Crear `usuarios/grpc` con:
- Endpoints de autenticación
- Middleware de autorización
- Proto definitions

### 2. Integración Frontend Iced
- Pantalla de login
- Almacenar token en estado
- Incluir token en requests gRPC

### 3. Integración con Reservas
- Añadir `usuario_id` a la entidad Reserva
- Solo usuarios autenticados pueden reservar
- Filtrar "Mis reservas" por usuario actual

### 4. PostgreSQL (opcional)
- Crear `PostgresUsuarioRepository`
- Migrations para tabla usuarios
- Índice único en email

---

## 💡 Tips de Uso

### 1. Seguridad de JWT Secret

**⚠️ IMPORTANTE:** En producción, usa variable de entorno:

```rust
// En jwt.rs, cambiar:
const JWT_SECRET: &str = env::var("JWT_SECRET")
    .expect("JWT_SECRET debe estar configurado");
```

### 2. Configurar Expiración de Token

```rust
// En jwt.rs
const TOKEN_EXPIRATION_HOURS: i64 = 24; // Cambiar según necesidad
```

### 3. Crear Usuario Admin al Inicio

```rust
// En main.rs del backend
if repo.listar().await?.is_empty() {
    auth_service.register(
        "Admin".to_string(),
        "admin@domain.com".to_string(),
        "changeme123".to_string(),
        Some(Rol::Admin),
    ).await?;
}
```

### 4. Validar Token en Middleware

```rust
// Ejemplo para Axum
async fn auth_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = JwtService::validate_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Agregar claims al request
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}
```

---

## 📚 Recursos

- [Argon2 - Password Hashing](https://github.com/P-H-C/phc-winner-argon2)
- [JWT - JSON Web Tokens](https://jwt.io/)
- [Arquitectura Hexagonal](https://netflixtechblog.com/ready-for-changes-with-hexagonal-architecture-b315ec967749)

---

## ✅ Estado Actual

- ✅ Domain layer completo con validaciones
- ✅ Auth layer con JWT y Argon2
- ✅ Application layer con AuthService y UsuarioService
- ✅ Infrastructure layer con FileUsuarioRepository
- ✅ 50+ tests unitarios
- ✅ Compilación exitosa
- ⏳ gRPC server (pendiente)
- ⏳ Integración con Iced (pendiente)
- ⏳ Proto definitions (pendiente)

**Sistema funcional y listo para crecer incrementalmente** 🚀
