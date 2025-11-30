# 🚀 Guía de Inicio Rápido

## Requisitos Previos

- Rust 1.70 o superior
- Cargo

## 1️⃣ Iniciar el Servidor

```bash
cargo run -p app
```

El servidor iniciará:
- **REST API** en `http://localhost:3000`
- **gRPC** en `http://localhost:50051`

Al iniciar por primera vez, se crea automáticamente:
- Usuario admin: `admin@reservas.com` / `admin123`
- Archivos de datos en `data/usuarios.json` y `data/salas.json`

## 2️⃣ Probar con CLI

### Login
```bash
cargo run -p app-cli -- login --email admin@reservas.com --password admin123
```

Guarda el token que te devuelve.

### Listar salas
```bash
cargo run -p app-cli -- sala --token "TU_TOKEN_AQUI" listar
```

### Crear sala
```bash
cargo run -p app-cli -- sala --token "TU_TOKEN_AQUI" crear --nombre "Sala 1" --capacidad 20
```

## 3️⃣ Aplicaciones Desktop

### Iced (Recomendado)
```bash
cargo run -p app-desktop-iced
```
- Login automático
- UI nativa más completa

### Dioxus
```bash
cargo run -p app-desktop-dioxus
```
- Requiere login manual
- Sintaxis React-like

### Slint
```bash
cargo run -p app-desktop-slint
```
- Login automático
- UI declarativa

### Tauri (Ejemplo REST)
```bash
cd crates/app-desktop-tauri
cargo tauri dev
```
- Usa REST en lugar de gRPC
- Ejemplo de integración híbrida

## 4️⃣ TUI (Terminal UI)

```bash
cargo run -p app-tui
```

Interfaz interactiva en terminal:
1. Ingresa credenciales (Tab para cambiar campo)
2. Enter para login
3. Navega con teclado

## 🔑 Credenciales por Defecto

| Usuario | Email | Contraseña | Rol |
|---------|-------|------------|-----|
| Admin | `admin@reservas.com` | `admin123` | Admin |
| David | `hola@david.com` | (configurada) | Admin |

## 🧪 Probar con grpcurl

### Login
```bash
grpcurl -plaintext -d '{
  "email": "admin@reservas.com",
  "password": "admin123"
}' localhost:50051 usuario.UsuarioService/Login
```

### Listar salas
```bash
grpcurl -plaintext \
  -H "authorization: Bearer TU_TOKEN" \
  -d '{}' \
  localhost:50051 sala.SalaService/ListarSalas
```

## 🧪 Probar con curl (REST)

### Login
```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@reservas.com","password":"admin123"}'
```

### Listar salas
```bash
curl http://localhost:3000/api/salas \
  -H "Authorization: Bearer TU_TOKEN"
```

## ❓ Solución de Problemas

### Error: "connection refused"
- Verifica que el servidor esté corriendo: `cargo run -p app`

### Error: "Email o contraseña incorrectos"
- Verifica que estés usando: `admin@reservas.com` (no `admin@example.com`)

### Error: "Token inválido o expirado"
- Genera un nuevo token con el comando `login`

## 📚 Siguiente Paso

Lee la [Documentación de API](API.md) para ver todos los endpoints disponibles.
