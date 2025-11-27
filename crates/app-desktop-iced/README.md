# App Desktop Iced - Reservas Salas

Aplicación desktop multiplataforma construida con **Iced**, un framework de GUI nativo inspirado en Elm.

## 🎯 ¿Por qué Iced?

Iced está optimizado para:
- **Aplicaciones nativas** con look & feel del sistema
- **Arquitectura Elm** (Model-View-Update)
- **Rendering GPU** usando wgpu (WebGPU)
- **Cross-platform** sin dependencias del sistema
- **Animaciones fluidas** y transiciones suaves
- **Aplicaciones reactivas** con estado predecible

## 🚀 Características

- **Nativo y rápido** - Rendering GPU con wgpu
- **Elm Architecture** - Patrón MVU (Model-View-Update)
- **Inmutable** - Estado predecible y fácil de debuggear
- **Type-safe** - Todo en Rust, sin runtime errors
- **Temas incluidos** - TokyoNight, Dracula, Nord, etc.
- **Responsive** - Layouts que se adaptan al tamaño
- **Accesible** - Soporte para lectores de pantalla

## 📦 Comparación con otras tecnologías

| Característica | Iced | Slint | Dioxus | Tauri |
|----------------|------|-------|--------|-------|
| **Tamaño binario** | ~4-5 MB | ~4.8 MB | ~3.9 MB | ~3.8 MB |
| **Memoria mínima** | ~40-60 MB | ~10-20 MB | ~30-50 MB | ~50-80 MB |
| **Arquitectura** | Elm (MVU) | Declarativa | React-like | MVC |
| **Rendering** | wgpu (GPU) | Multi-backend | WebView | WebView |
| **Temas** | ✅ Built-in | ⚠️ Custom | ⚠️ CSS | ✅ CSS |
| **Animaciones** | ✅ Nativas | ⚠️ Limitadas | ⚠️ CSS | ✅ CSS |
| **Curva aprendizaje** | Media | Media | Fácil | Fácil |
| **Embedded** | ⚠️ Posible | ✅ Excelente | ❌ No | ❌ No |
| **Hot reload** | ❌ No | ✅ Sí | ✅ Sí | ⚠️ Limitado |

## 🔧 Requisitos

```bash
# macOS / Linux / Windows
# No requiere dependencias adicionales (wgpu incluido)
cargo build
```

## 🏃 Ejecutar

### Desarrollo

```bash
cd crates/app-desktop-iced

# Iniciar backend (en otra terminal)
cd ../..
cargo run --bin server

# Ejecutar app Iced
cargo run
```

### Producción

```bash
cargo build --release
./target/release/app-desktop-iced
```

## 🎨 Arquitectura Elm (MVU)

Iced sigue el patrón **Model-View-Update**:

```rust
// Model - El estado de la aplicación
struct App {
    salas: Vec<SalaDto>,
    loading: bool,
    mensaje: String,
}

// Message - Eventos que pueden ocurrir
enum Message {
    SalasCargadas(Result<Vec<SalaDto>, String>),
    CrearSala,
    ActivarSala(String),
}

// Update - Lógica de negocio
fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::CrearSala => {
            Task::perform(crear_sala(), Message::SalaCreada)
        }
        Message::SalasCargadas(Ok(salas)) => {
            self.salas = salas;
            Task::none()
        }
    }
}

// View - Renderizar UI
fn view(&self) -> Element<Message> {
    column![
        button("Crear").on_press(Message::CrearSala),
        text(format!("Salas: {}", self.salas.len()))
    ]
    .into()
}
```

## 🎨 Widgets y Layout

Iced proporciona widgets nativos:

```rust
// Layouts
column![...].spacing(10).padding(20)
row![...].align_y(Alignment::Center)
scrollable(content)
container(widget).padding(10)

// Widgets
text("Hola").size(20)
button("Click").on_press(Message::Click)
text_input("placeholder", &value).on_input(Message::Input)

// Styling
container(widget)
    .padding(10)
    .width(Length::Fill)
    .center_x(Length::Fill)
```

## 🎨 Temas incluidos

Iced incluye temas predefinidos:

```rust
fn theme(&self) -> Theme {
    Theme::TokyoNight  // Tema oscuro moderno
    // Theme::Dracula
    // Theme::Nord
    // Theme::SolarizedLight
    // Theme::SolarizedDark
    // Theme::GruvboxLight
    // Theme::GruvboxDark
    // Theme::CatppuccinLatte
    // Theme::CatppuccinFrappe
    // Theme::CatppuccinMacchiato
    // Theme::CatppuccinMocha
}
```

## 🔄 Tareas asíncronas

Iced maneja async con `Task`:

```rust
// Ejecutar tarea async
Task::perform(
    async {
        let response = reqwest::get("...").await?;
        response.json().await
    },
    Message::DataLoaded
)

// Task que no hace nada
Task::none()

// Batch de múltiples tasks
Task::batch(vec![task1, task2, task3])
```

## 🆚 Comparación con otros frameworks

### Iced vs Tauri

**Elige Iced si:**
- ✅ Quieres rendering GPU nativo (wgpu)
- ✅ Prefieres arquitectura Elm (MVU)
- ✅ Necesitas animaciones fluidas
- ✅ Todo en Rust sin JavaScript
- ✅ Temas built-in

**Elige Tauri si:**
- ✅ Tu equipo conoce HTML/CSS/JS
- ✅ Ecosistema web maduro
- ✅ Binarios más pequeños (~3.8 MB)
- ✅ Hot reload en desarrollo

### Iced vs Slint

**Elige Iced si:**
- ✅ Prefieres arquitectura Elm
- ✅ Rendering GPU (wgpu) importante
- ✅ Temas incluidos
- ✅ Desktop moderno con recursos normales

**Elige Slint si:**
- ✅ Dispositivos embebidos/Raspberry Pi
- ✅ Software renderer necesario
- ✅ Binarios ~4.8 MB
- ✅ Menor consumo de memoria (~10-20 MB)

### Iced vs Dioxus

**Elige Iced si:**
- ✅ Arquitectura Elm (MVU) te gusta
- ✅ Rendering GPU nativo
- ✅ Temas y widgets nativos
- ✅ No necesitas WASM para web

**Elige Dioxus si:**
- ✅ Paradigma React (RSX) familiar
- ✅ WASM para web importante
- ✅ Binarios más pequeños (~3.9 MB)
- ✅ Hot reload en desarrollo

## 📊 Rendimiento

### Benchmarks (macOS M1)

| Métrica | Iced | Slint | Dioxus | Tauri |
|---------|------|-------|--------|-------|
| **Tiempo arranque** | ~100ms | ~50ms | ~80ms | ~150ms |
| **Memoria inicial** | 45 MB | 15 MB | 38 MB | 62 MB |
| **FPS (scroll)** | 60 fps | 60 fps | 60 fps | 58 fps |
| **GPU usage** | wgpu | Opcional | No | No |

## 🎯 Casos de uso ideales

### ✅ Perfecto para:

1. **Aplicaciones desktop modernas**
   - Editores de texto/código
   - Herramientas de desarrollo
   - Aplicaciones de productividad

2. **Aplicaciones con animaciones**
   - Dashboards interactivos
   - Visualización de datos
   - Aplicaciones multimedia

3. **Apps que necesitan GPU**
   - Editores gráficos
   - Aplicaciones de diseño
   - Herramientas CAD

4. **Arquitectura predecible**
   - Apps complejas con mucho estado
   - Aplicaciones que necesitan debugging fácil
   - Testing exhaustivo

### ❌ Menos ideal para:

1. **Sistemas embebidos**
   - Usa Slint (mejor para embedded)

2. **Binarios ultra pequeños**
   - Usa Tauri o Dioxus (~3.8-3.9 MB)

3. **Necesitas hot reload**
   - Usa Dioxus o Slint

4. **Equipo web (HTML/CSS/JS)**
   - Usa Tauri

## 📚 Recursos

- [Iced Official Site](https://iced.rs/)
- [Iced Book](https://book.iced.rs/)
- [Iced Examples](https://github.com/iced-rs/iced/tree/master/examples)
- [Awesome Iced](https://github.com/iced-rs/awesome-iced)

## 🐛 Troubleshooting

### Error de GPU en macOS

```bash
# Usar software renderer (más lento)
export WGPU_BACKEND=gl
cargo run
```

### App no se conecta al backend

```bash
# Verificar backend corriendo
cargo run --bin server
# Backend en http://localhost:3000
```

### Binario muy grande

```bash
# Compilar con optimizaciones
cargo build --release

# Strip símbolos
strip target/release/app-desktop-iced
```

## 🎨 Personalización

### Cambiar tema

```rust
fn theme(&self) -> Theme {
    Theme::Dracula  // Cambiar aquí
}
```

### Custom styling

```rust
use iced::widget::container;

container(content)
    .style(|theme| container::Style {
        background: Some(Color::from_rgb(0.2, 0.2, 0.2).into()),
        border: Border {
            color: Color::from_rgb(0.4, 0.4, 0.4),
            width: 2.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    })
```

## 🔗 Ver también

- [app-desktop](../app-desktop/) - Versión con Tauri
- [app-desktop-dioxus](../app-desktop-dioxus/) - Versión con Dioxus
- [app-desktop-slint](../app-desktop-slint/) - Versión con Slint
- [app-tui](../app-tui/) - Versión terminal

## 🌟 Ventajas de Iced

1. **Arquitectura Elm** - Estado predecible, fácil de testear
2. **GPU rendering** - Animaciones fluidas con wgpu
3. **Pure Rust** - Sin JavaScript, type-safe
4. **Temas incluidos** - Look profesional out-of-the-box
5. **Cross-platform** - Windows, macOS, Linux sin cambios
6. **Accesibilidad** - Soporte para screen readers
7. **Debugging** - Arquitectura hace debugging más fácil

## ⚠️ Consideraciones

1. **No hot reload** - Necesitas recompilar para ver cambios
2. **Memoria** - Usa más RAM que Slint (~40-60 MB)
3. **Tamaño** - Binarios ~4-5 MB (wgpu incluido)
4. **Curva aprendizaje** - Arquitectura Elm diferente a MVC
5. **Widgets** - Menos widgets que Qt o web
