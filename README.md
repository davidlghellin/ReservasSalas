# 🏢 Sistema de Reservas de Salas

Sistema completo de gestión de salas con autenticación JWT, implementado en Rust siguiendo principios de **Vertical Slice Architecture** y **Clean Architecture**.

## 📋 Tabla de Contenidos

- [Características](#características)
- [Arquitectura](#arquitectura)
- [Tecnologías](#tecnologías)
- [Aplicaciones Cliente](#aplicaciones-cliente)
- [Inicio Rápido](#inicio-rápido)
- [Credenciales de Prueba](#credenciales-de-prueba)
- [API](#api)

## ✨ Características

- ✅ **Autenticación JWT** - Sistema completo de usuarios con roles (Admin/Usuario)
- ✅ **gRPC + REST** - Dual API (gRPC en puerto 50051, REST en puerto 3000)
- ✅ **Múltiples Clientes** - Desktop (Iced, Dioxus, Slint, Tauri), CLI, TUI, Web
- ✅ **CRUD Completo** - Gestión de salas con validaciones
- ✅ **Persistencia** - Repositorio basado en archivos JSON
- ✅ **Clean Architecture** - Separación clara de responsabilidades
- ✅ **Tests Unitarios** - Cobertura de lógica de negocio

## 🏗️ Arquitectura

```
crates/
├── app/                              # Servidor principal (REST + gRPC)
├── app-cli/                          # Cliente CLI con gRPC
├── app-tui/                          # Cliente TUI (Terminal UI) con gRPC
├── app-web/                          # Aplicación web con templates
├── app-desktop-iced/                 # Cliente desktop con Iced + gRPC
├── app-desktop-dioxus/               # Cliente desktop con Dioxus + gRPC
├── app-desktop-slint/                # Cliente desktop con Slint + gRPC
├── app-desktop-tauri/                # Cliente desktop con Tauri + REST (ejemplo)
└── features/
    ├── salas/
    │   ├── domain/                   # Lógica de negocio pura
    │   ├── application/              # Casos de uso
    │   ├── infrastructure/           # Repositorio (archivos JSON)
    │   ├── api/                      # REST API
    │   └── grpc/                     # gRPC Server con autenticación JWT
    └── usuarios/
        ├── domain/                   # Entidades de usuario
        ├── application/              # Servicios de autenticación
        ├── infrastructure/           # Repositorio de usuarios
        ├── auth/                     # JWT y hash de contraseñas
        ├── api/                      # REST endpoints
        └── grpc/                     # gRPC Server con autenticación JWT
```

## 🛠️ Tecnologías

### Backend
- **Rust** - Lenguaje de programación
- **Axum** - Framework web REST
- **Tonic** - Framework gRPC
- **Tokio** - Runtime asíncrono
- **Argon2** - Hash de contraseñas
- **jsonwebtoken** - JWT tokens

### Frontend/Clientes
- **Iced** - UI nativa con Elm architecture
- **Dioxus** - UI reactiva con sintaxis React-like
- **Slint** - UI declarativa con lenguaje propio
- **Tauri** - Desktop híbrido (Rust + HTML/CSS/JS)
- **Ratatui** - Terminal UI

## 📱 Aplicaciones Cliente

### Desktop con gRPC + JWT

| App | Framework | Protocolo | Estado |
|-----|-----------|-----------|--------|
| **Iced** | Iced GUI | gRPC :50051 | ✅ Completado |
| **Dioxus** | Dioxus | gRPC :50051 | ✅ Completado |
| **Slint** | Slint UI | gRPC :50051 | ✅ Completado |

### CLI/TUI con gRPC + JWT

| App | Framework | Protocolo | Estado |
|-----|-----------|-----------|--------|
| **CLI** | Clap | gRPC :50051 | ✅ Completado |
| **TUI** | Ratatui | gRPC :50051 | ✅ Completado |

### Otros

| App | Framework | Protocolo | Notas |
|-----|-----------|-----------|-------|
| **Tauri** | Tauri | REST :3000 | Ejemplo de REST con JWT |
| **Web** | Axum + Askama | Server-side | Templates HTML |

## 🚀 Inicio Rápido

### 1. Iniciar el servidor

```bash
# Inicia REST API (puerto 3000) y gRPC (puerto 50051)
cargo run -p app
```

El servidor creará automáticamente:
- Usuario administrador por defecto
- Archivo de datos en `data/usuarios.json` y `data/salas.json`

### 2. Usar el CLI

```bash
# Login para obtener token JWT
cargo run -p app-cli -- login --email admin@reservas.com --password admin123

# Listar salas (requiere token del paso anterior)
cargo run -p app-cli -- sala --token "YOUR_TOKEN" listar

# Crear sala
cargo run -p app-cli -- sala --token "YOUR_TOKEN" crear --nombre "Sala 1" --capacidad 20

# Activar sala
cargo run -p app-cli -- sala --token "YOUR_TOKEN" activar --id "SALA_ID"
```

### 3. Usar aplicaciones desktop

```bash
# Iced (login automático con credenciales por defecto)
cargo run -p app-desktop-iced

# Dioxus
cargo run -p app-desktop-dioxus

# Slint (login automático)
cargo run -p app-desktop-slint

# Tauri
cd crates/app-desktop-tauri
cargo tauri dev
```

### 4. Usar TUI (Terminal UI)

```bash
cargo run -p app-tui
```

## 🔑 Credenciales de Prueba

**Email:** `admin@reservas.com`
**Contraseña:** `admin123`
**Rol:** Admin

**Usuario alternativo:**
**Email:** `hola@david.com`
**Contraseña:** (la que hayas configurado)
**Rol:** Admin

## 📡 API

### gRPC (Puerto 50051)

```bash
# Login
grpcurl -plaintext -d '{
  "email": "admin@reservas.com",
  "password": "admin123"
}' localhost:50051 usuario.UsuarioService/Login

# Listar salas (requiere token en metadata)
grpcurl -plaintext \
  -H "authorization: Bearer YOUR_TOKEN" \
  -d '{}' \
  localhost:50051 sala.SalaService/ListarSalas

# Crear sala
grpcurl -plaintext \
  -H "authorization: Bearer YOUR_TOKEN" \
  -d '{
    "nombre": "Sala de Conferencias",
    "capacidad": 20
  }' \
  localhost:50051 sala.SalaService/CrearSala
```

### REST API (Puerto 3000)

```bash
# Login
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@reservas.com",
    "password": "admin123"
  }'

# Listar salas (requiere token)
curl http://localhost:3000/api/salas \
  -H "Authorization: Bearer YOUR_TOKEN"

# Crear sala
curl -X POST http://localhost:3000/api/salas \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "nombre": "Sala de Conferencias",
    "capacidad": 20
  }'
```

## 🧪 Tests

```bash
# Ejecutar todos los tests
cargo test

# Tests de un módulo específico
cargo test -p salas-domain
cargo test -p usuarios-application
```

## 📄 Reglas de Negocio

### Salas
- El nombre no puede estar vacío
- El nombre no puede exceder 100 caracteres
- La capacidad debe estar entre 1 y 1000
- Las salas se crean activas por defecto
- Solo usuarios autenticados pueden gestionar salas

### Usuarios
- Email debe ser válido y único
- Contraseña mínimo 8 caracteres
- Roles: Admin o Usuario
- Usuarios se crean activos por defecto
- Solo Admin puede listar usuarios

## 📚 Documentación Adicional

- [GRPC_INTEGRATION_COMPLETE.md](GRPC_INTEGRATION_COMPLETE.md) - Integración completa de gRPC
- [ICED_AUTH_INTEGRATION.md](ICED_AUTH_INTEGRATION.md) - Autenticación en Iced
- [INTEGRATION_COMPLETE.md](INTEGRATION_COMPLETE.md) - Estado de integración
- [USUARIOS_SYSTEM_CREATED.md](USUARIOS_SYSTEM_CREATED.md) - Sistema de usuarios

## 📝 Notas

- Los datos se persisten en archivos JSON en la carpeta `data/`
- El servidor crea automáticamente un usuario admin al iniciar
- Todas las aplicaciones desktop con gRPC requieren que el servidor esté corriendo
- El token JWT expira según la configuración (por defecto: 24 horas)
