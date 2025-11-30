# 📚 Documentación del Sistema de Reservas

## 📖 Guías Principales

- **[Inicio Rápido](QUICK_START.md)** - Empieza aquí si es tu primera vez
- **[API](API.md)** - Referencia completa de endpoints REST y gRPC
- **[Roadmap Detallado](ROADMAP_DETALLADO.md)** - Plan de desarrollo futuro

## 📱 Aplicaciones

### Desktop (gRPC + JWT)

| App | Comando | Login |
|-----|---------|-------|
| **Iced** | `cargo run -p app-desktop-iced` | Automático ⭐ |
| **Dioxus** | `cargo run -p app-desktop-dioxus` | Manual |
| **Slint** | `cargo run -p app-desktop-slint` | Automático |
| **Tauri** | `cd crates/app-desktop-tauri && cargo tauri dev` | Manual (REST) |

### Terminal (gRPC + JWT)

| App | Comando |
|-----|---------|
| **CLI** | `cargo run -p app-cli -- login --email admin@reservas.com --password admin123` |
| **TUI** | `cargo run -p app-tui` |

## 🔑 Credenciales

```
Email:    admin@reservas.com
Password: admin123
```

## 📁 Estructura de Documentación

```
docs/
├── QUICK_START.md          # Tutorial paso a paso
├── API.md                  # Referencia REST y gRPC
├── ROADMAP_DETALLADO.md    # Plan de desarrollo
├── apps/                   # Docs específicas (Tauri, comparaciones)
└── historico/              # Documentación del desarrollo
```

## 📜 Documentación Histórica

La carpeta [historico/](historico/) contiene documentación del proceso de desarrollo histórico.

> **Nota:** Los documentos históricos pueden estar desactualizados. Para información actual, consulta las guías principales.

## 🆘 Ayuda

### Primeros pasos
1. Lee [Inicio Rápido](QUICK_START.md)
2. Inicia el servidor: `cargo run -p app`
3. Prueba el CLI: `cargo run -p app-cli -- login --email admin@reservas.com --password admin123`

### Problemas comunes

**Error de conexión**
- Verifica que el servidor esté corriendo
- Puertos: REST=3000, gRPC=50051

**Credenciales incorrectas**
- Email: `admin@reservas.com` (no `@example.com`)
- Password: `admin123`

**Token expirado**
- Genera un nuevo token con el comando `login`

### Más información

- 📖 [README principal](../README.md)
- 📡 [Documentación de API](API.md)
- 🚀 [Guía de inicio rápido](QUICK_START.md)
