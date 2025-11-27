# App Desktop Dioxus - Reservas Salas

Aplicación desktop multiplataforma construida con **Dioxus**, el framework de UI nativo para Rust.

## 🚀 Características

- **100% Rust** - Sin JavaScript, todo el código es Rust
- **Nativo y rápido** - Usa WebView nativo del sistema
- **Reactivo** - Sistema de señales similar a React hooks
- **Hot reload** - Recarga automática en desarrollo
- **Multiplataforma** - Windows, macOS, Linux desde el mismo código
- **Pequeño** - Binarios más pequeños que Electron

## 📦 Diferencias con Tauri

| Característica | Dioxus | Tauri |
|----------------|--------|-------|
| Frontend | Rust (RSX) | HTML/CSS/JS |
| Backend | Rust | Rust |
| Comunicación | Directo | IPC + Serde |
| Hot reload | ✅ Integrado | ⚠️ Limitado |
| Curva aprendizaje | Rust | Web (HTML/JS) + Rust |
| Tamaño binario | ~8-10 MB | ~10-12 MB |
| Ecosistema UI | Dioxus components | Web components |

## 🔧 Requisitos

```bash
# Instalar Dioxus CLI
cargo install dioxus-cli

# Dependencias del sistema (igual que Tauri)
# macOS: ya incluidas
# Linux: libwebkit2gtk-4.1-dev
# Windows: WebView2 (pre-instalado en Windows 11)
```

## 🏃 Ejecutar

### Desarrollo (con hot reload)

```bash
cd crates/app-desktop-dioxus

# Iniciar backend (en otra terminal)
cd ../..
cargo run --bin server

# Ejecutar app Dioxus
dx serve --hot-reload
```

### Producción

```bash
cd crates/app-desktop-dioxus

# Compilar binario release
cargo build --release

# Ejecutar
../../target/release/app-desktop-dioxus
```

## 📊 Comparación de rendimiento

### Tamaño de binarios (Release, stripped)

| Plataforma | Dioxus | Tauri |
|------------|--------|-------|
| macOS | ~8 MB | ~10 MB |
| Linux | ~9 MB | ~11 MB |
| Windows | ~9 MB | ~11 MB |

### Tiempo de inicio

- **Dioxus**: ~50-100ms
- **Tauri**: ~100-200ms

### Uso de memoria

- **Dioxus**: ~30-50 MB
- **Tauri**: ~50-80 MB

## 🎨 Estructura del código

```
app-desktop-dioxus/
├── Cargo.toml           # Dependencias
├── Dioxus.toml          # Configuración Dioxus
├── assets/
│   └── style.css        # Estilos CSS
└── src/
    └── main.rs          # App principal (RSX components)
```

## 📝 Ejemplo de código RSX

```rust
rsx! {
    div { class: "container",
        h1 { "Hola Dioxus!" }

        button {
            onclick: move |_| println!("Click!"),
            "Hacer click"
        }

        for item in items.read().iter() {
            div { "{item}" }
        }
    }
}
```

## 🔄 Estado reactivo

Dioxus usa un sistema de señales:

```rust
// Signal mutable
let mut count = use_signal(|| 0);

// Leer valor
println!("Count: {}", *count.read());

// Modificar valor
count.set(42);

// Actualizar basado en valor actual
count.with_mut(|c| *c += 1);
```

## 🌐 Comunicación con backend

La app usa `reqwest` para comunicarse con la API REST:

```rust
async fn listar_salas() -> Result<Vec<SalaDto>, String> {
    let response = reqwest::get("http://localhost:3000/api/salas")
        .await
        .map_err(|e| format!("Error: {}", e))?;

    response.json().await
        .map_err(|e| format!("Error: {}", e))
}
```

## 🏗️ Compilar para distribución

### macOS

```bash
dx build --release
# Binario en: target/release/app-desktop-dioxus
```

### Linux

```bash
dx build --release --platform desktop
```

### Windows

```bash
dx build --release --platform desktop
```

## 🆚 ¿Cuándo usar Dioxus vs Tauri?

### Usar Dioxus si:
- ✅ Quieres escribir todo en Rust
- ✅ Prefieres componentes nativos a HTML/CSS
- ✅ Necesitas hot reload rápido
- ✅ Tu equipo ya conoce Rust bien
- ✅ Quieres aprender un framework moderno de UI en Rust

### Usar Tauri si:
- ✅ Tu equipo conoce HTML/CSS/JS
- ✅ Quieres reutilizar componentes web existentes
- ✅ Necesitas un ecosistema más maduro
- ✅ Prefieres separación clara frontend/backend
- ✅ Quieres usar frameworks web (React, Vue, Svelte)

## 📚 Recursos

- [Dioxus Docs](https://dioxuslabs.com/)
- [Dioxus Examples](https://github.com/DioxusLabs/dioxus/tree/main/examples)
- [Awesome Dioxus](https://github.com/DioxusLabs/awesome-dioxus)

## 🐛 Troubleshooting

### Hot reload no funciona

```bash
# Limpiar y reiniciar
cargo clean
dx serve --hot-reload
```

### Error de WebView en Linux

```bash
# Instalar WebKit
sudo apt-get install libwebkit2gtk-4.1-dev
```

### App no se conecta al backend

Asegúrate de que el backend está corriendo:

```bash
cargo run --bin server
# Backend en http://localhost:3000
```

## 🔗 Ver también

- [app-desktop](../app-desktop/) - Versión con Tauri
- [app-web](../app-web/) - Versión web con SSR
- [app-tui](../app-tui/) - Versión terminal
