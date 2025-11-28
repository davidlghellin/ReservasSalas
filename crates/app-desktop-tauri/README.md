# App Desktop - Tauri

Aplicación de escritorio multiplataforma para la gestión de salas, construida con Tauri y reutilizando la lógica de negocio existente.

## Arquitectura

```
┌──────────────────────────┐
│    Frontend (HTML/JS)    │
│   ▪ HTML5 + CSS3         │
│   ▪ Vanilla JavaScript   │
└────────────┬─────────────┘
             │
   Tauri IPC Bridge
             │
┌────────────▼─────────────┐
│  Tauri Commands (Rust)   │  ◄── Este crate
│   ▪ crear_sala           │
│   ▪ listar_salas         │
│   ▪ obtener_sala         │
│   ▪ activar_sala         │
│   ▪ desactivar_sala      │
└────────────┬─────────────┘
             │
┌────────────▼─────────────┐
│    SalaService           │  ◄── Application layer
└────────────┬─────────────┘
             │
┌────────────▼─────────────┘
│       Domain             │
└──────────────────────────┘
```

## Características

✅ **Multiplataforma**: Windows, macOS, Linux
✅ **Ligero**: ~600KB vs Electron (~100MB)
✅ **Nativo**: Usa el WebView del sistema operativo
✅ **Reutilización**: Comparte lógica con REST API y gRPC
✅ **Offline**: Funciona completamente sin conexión

## Requisitos

- Rust 1.70+
- Tauri CLI (se instala automáticamente)
- Dependencias del sistema según OS:
  - **macOS**: Xcode Command Line Tools
  - **Linux**: `webkit2gtk`, `libappindicator3`
  - **Windows**: WebView2

## Instalación de dependencias

### macOS
```bash
xcode-select --install
```

### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libxdo-dev \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

### Windows
WebView2 viene preinstalado en Windows 11. Para Windows 10, se descarga automáticamente.

## Compilar y ejecutar

### Modo desarrollo
```bash
cd crates/app-desktop-tauri/src-tauri
cargo run
```

### Modo release (optimizado)
```bash
cd crates/app-desktop-tauri/src-tauri
cargo build --release

# El binario estará en:
../../../target/release/app-desktop-tauri
```

### Ejecutar directamente
```bash
# Desde la raíz del proyecto
./target/release/app-desktop-tauri
```

## Estructura del proyecto

```
crates/app-desktop/
├── src/                    # Frontend (HTML/CSS/JS)
│   ├── index.html         # Página principal
│   ├── styles.css         # Estilos
│   └── main.js            # Lógica del frontend
│
├── src-tauri/             # Backend Rust (Tauri)
│   ├── src/
│   │   ├── main.rs        # Entry point
│   │   ├── lib.rs         # Configuración Tauri
│   │   └── commands.rs    # Tauri commands
│   │
│   ├── Cargo.toml         # Dependencias Rust
│   ├── tauri.conf.json    # Configuración Tauri
│   ├── build.rs           # Build script
│   └── icons/             # Iconos de la app
│
└── README.md              # Este archivo
```

## Tauri Commands

Los commands son funciones Rust que el frontend puede invocar:

### Ejemplo desde JavaScript:

```javascript
import { invoke } from '@tauri-apps/api/core';

// Crear una sala
const sala = await invoke('crear_sala', {
    request: {
        nombre: "Sala Conferencias",
        capacidad: 50
    }
});

// Listar salas
const salas = await invoke('listar_salas');

// Obtener sala por ID
const sala = await invoke('obtener_sala', { id: "abc-123" });

// Activar sala
await invoke('activar_sala', { id: "abc-123" });

// Desactivar sala
await invoke('desactivar_sala', { id: "abc-123" });
```

### Implementación en Rust:

Todos los commands están definidos en `src-tauri/src/commands.rs` y utilizan el `SalaService` de la capa de aplicación:

```rust
#[tauri::command]
pub async fn crear_sala(
    request: CrearSalaRequest,
    service: State<'_, SharedSalaService>,
) -> Result<SalaDto, String> {
    service
        .crear_sala(request.nombre, request.capacidad)
        .await
        .map(|sala| sala.into())
        .map_err(|e| e.to_string())
}
```

## Frontend

El frontend está construido con tecnologías web estándar:

- **HTML5**: Estructura semántica
- **CSS3**: Estilos modernos con gradientes y animaciones
- **Vanilla JavaScript**: Sin frameworks, ligero y rápido

### Características del UI:

- 🎨 Diseño moderno con gradientes
- 📱 Responsive design
- ✨ Animaciones suaves
- 🔔 Notificaciones toast
- 🃏 Cards con hover effects
- ⚡ Actualización en tiempo real

## Modificar la aplicación

### Añadir nuevos commands:

1. **Agregar método al trait en Application layer** (si es necesario)
2. **Implementar el command en `src-tauri/src/commands.rs`**:
```rust
#[tauri::command]
pub async fn eliminar_sala(
    id: String,
    service: State<'_, SharedSalaService>,
) -> Result<(), String> {
    service
        .eliminar_sala(&id)
        .await
        .map_err(|e| e.to_string())
}
```

3. **Registrar el command en `src-tauri/src/lib.rs`**:
```rust
.invoke_handler(tauri::generate_handler![
    commands::crear_sala,
    commands::listar_salas,
    commands::obtener_sala,
    commands::activar_sala,
    commands::desactivar_sala,
    commands::eliminar_sala,  // ← Nuevo
])
```

4. **Usar desde JavaScript**:
```javascript
await invoke('eliminar_sala', { id: "abc-123" });
```

### Modificar el frontend:

Edita los archivos en `src/`:
- `index.html` - Estructura HTML
- `styles.css` - Estilos y diseño
- `main.js` - Lógica de la aplicación

## Crear instalador

```bash
cd crates/app-desktop-tauri/src-tauri

# Genera instaladores para tu plataforma
cargo tauri build
```

Los instaladores se crean en:
- **macOS**: `target/release/bundle/dmg/`
- **Windows**: `target/release/bundle/msi/`
- **Linux**: `target/release/bundle/deb/` y `/appimage/`

## Ventajas de Tauri

| Característica | Tauri | Electron |
|---------------|-------|----------|
| Tamaño del binario | ~600 KB | ~100 MB |
| Uso de memoria | ~40 MB | ~200 MB |
| Backend | Rust | Node.js |
| WebView | Sistema | Chromium |
| Seguridad | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |
| Rendimiento | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |

## Comparación con otras apps del proyecto

| App | Tecnología | Uso |
|-----|------------|-----|
| `app` | HTTP + gRPC Server | Servidor backend para APIs |
| `app-web` | Axum + Askama | Aplicación web con SSR |
| `app-cli` | CLI | Herramienta de línea de comandos |
| `app-tui` | Ratatui | Terminal UI interactiva |
| **`app-desktop`** | **Tauri** | **Aplicación de escritorio nativa** |

Todas comparten el mismo código de dominio, aplicación e infraestructura.

## Troubleshooting

### Error: "failed to open icon"
Asegúrate de tener al menos un `icon.png` en `src-tauri/icons/`.

### Error: "webkit2gtk not found" (Linux)
Instala las dependencias del sistema:
```bash
sudo apt install libwebkit2gtk-4.1-dev
```

### La ventana no se abre
Verifica que no haya otro proceso usando los recursos. Intenta ejecutar en modo debug:
```bash
RUST_LOG=debug cargo run
```

### Cambios no se reflejan
Asegúrate de recompilar después de modificar código Rust:
```bash
cargo build --release
```

Para cambios en HTML/CSS/JS, solo necesitas recargar la app.

## Referencias

- [Tauri Documentation](https://tauri.app/)
- [Tauri API](https://tauri.app/v1/api/js/)
- [Rust Tauri](https://docs.rs/tauri/)
- [Tauri Examples](https://github.com/tauri-apps/tauri/tree/dev/examples)

## Próximos pasos

Posibles mejoras futuras:

- [ ] Añadir persistencia con SQLite
- [ ] Implementar sistema de reservas
- [ ] Agregar calendario visual
- [ ] Soporte para temas (claro/oscuro)
- [ ] Notificaciones del sistema
- [ ] Exportar reportes a PDF/Excel
- [ ] Sincronización con servidor REST/gRPC
- [ ] Soporte multi-idioma (i18n)
