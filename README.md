# 🏢 Sistema de Reservas de Salas

Sistema completo de gestión de salas con autenticación JWT, implementado en Rust con arquitectura limpia.

## 🚀 Inicio Rápido

```bash
# 1. Iniciar servidor (REST + gRPC)
cargo run -p app

# 2. Probar con CLI
cargo run -p app-cli -- login --email admin@reservas.com --password admin123

# 3. Usar aplicación desktop
cargo run -p app-desktop-iced
```

**📖 [Guía Completa de Inicio Rápido](docs/QUICK_START.md)**

## ✨ Características

- 🔐 **Autenticación JWT** - Sistema de usuarios con roles (Admin/Usuario)
- 🌐 **Dual API** - gRPC (puerto 50051) + REST (puerto 3000)
- 🖥️ **Múltiples Clientes** - Desktop (Iced, Dioxus, Slint, Tauri), CLI, TUI
- 💾 **Persistencia** - Repositorio basado en archivos JSON
- 🏗️ **Clean Architecture** - Separación clara de responsabilidades
- ✅ **Tests Unitarios** - Cobertura de lógica de negocio

## 📱 Aplicaciones Cliente

| Tipo | App | Framework | Protocolo | Estado |
|------|-----|-----------|-----------|--------|
| **Desktop** | Iced | Iced GUI | gRPC | ✅ |
| | Dioxus | Dioxus | gRPC | ✅ |
| | Slint | Slint UI | gRPC | ✅ |
| | Tauri | Tauri | REST | ✅ |
| **Terminal** | CLI | Clap | gRPC | ✅ |
| | TUI | Ratatui | gRPC | ✅ |
| **Web** | Web | Axum + Askama | Server-side | ✅ |

## 🏗️ Arquitectura

```
crates/
├── app/                          # Servidor (REST + gRPC)
├── app-{cli,tui}                 # Clientes terminal
├── app-desktop-{iced,dioxus,slint,tauri}
└── features/
    ├── salas/                    # Feature: Gestión de salas
    │   ├── domain/               # Lógica de negocio
    │   ├── application/          # Casos de uso
    │   ├── infrastructure/       # Persistencia
    │   ├── api/                  # REST endpoints
    │   └── grpc/                 # gRPC server
    └── usuarios/                 # Feature: Autenticación
        ├── domain/
        ├── application/
        ├── infrastructure/
        ├── auth/                 # JWT + Argon2
        ├── api/
        └── grpc/
```

## 🔑 Credenciales por Defecto

```
Email:    admin@reservas.com
Password: admin123
Rol:      Admin
```

## 📡 API

### gRPC (Puerto 50051)
```bash
# Login
grpcurl -plaintext -d '{
  "email": "admin@reservas.com",
  "password": "admin123"
}' localhost:50051 usuario.UsuarioService/Login

# Listar salas (requiere token)
grpcurl -plaintext \
  -H "authorization: Bearer TOKEN" \
  -d '{}' \
  localhost:50051 sala.SalaService/ListarSalas
```

### REST (Puerto 3000)
```bash
# Login
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@reservas.com","password":"admin123"}'

# Listar salas
curl http://localhost:3000/api/salas \
  -H "Authorization: Bearer TOKEN"
```

**📖 [Documentación Completa de API](docs/API.md)**

## 🛠️ Tecnologías

- **Rust** - Lenguaje de programación
- **Axum** - Framework web REST
- **Tonic** - Framework gRPC
- **Tokio** - Runtime asíncrono
- **Argon2** - Hash de contraseñas
- **jsonwebtoken** - Autenticación JWT
- **Iced/Dioxus/Slint/Tauri** - Frameworks UI

## 🧪 Tests

```bash
# Ejecutar todos los tests
cargo test

# Tests específicos
cargo test -p salas-domain
cargo test -p usuarios-application
```

## 📚 Documentación

- 📖 **[Inicio Rápido](docs/QUICK_START.md)** - Guía para empezar
- 📡 **[API](docs/API.md)** - Referencia completa de endpoints
- 🏛️ **[Arquitectura](docs/ARCHITECTURE.md)** - Diseño del sistema *(próximamente)*
- 📜 **[Histórico](docs/historico/)** - Documentación de desarrollo

## 📄 Reglas de Negocio

### Salas
- Nombre: No vacío, máximo 100 caracteres
- Capacidad: Entre 1 y 1000 personas
- Estado: Activa/Inactiva (activa por defecto)

### Usuarios
- Email: Formato válido, único
- Contraseña: Mínimo 8 caracteres
- Roles: Admin o Usuario
- Estado: Activo/Inactivo (activo por defecto)

## 🤝 Contribuir

Este es un proyecto de ejemplo educativo. Pull requests son bienvenidos.

## 📝 Licencia

MIT

---

**💡 Tip:** Empieza con la [Guía de Inicio Rápido](docs/QUICK_START.md) si es tu primera vez.
