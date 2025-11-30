# Ejemplos de Uso - salas-validation

Ejemplos prácticos de cómo usar `salas-validation` en diferentes contextos.

## 📱 Frontend: Iced (gRPC)

```rust
use iced::Task;
use salas_grpc::proto::CrearSalaRequest;
use salas_validation::ValidarSala;

Message::CrearSala => {
    let capacidad = self.nueva_capacidad.parse::<u32>().unwrap_or(0);

    let request = CrearSalaRequest {
        nombre: self.nuevo_nombre.clone(),
        capacidad,
    };

    // ✅ Validar antes de enviar
    if let Err(e) = request.validar() {
        self.mensaje = format!("❌ {}", e.mensaje_usuario());
        return Task::none();
    }

    // Request válido, enviar al backend
    Task::perform(crear_sala(request), Message::SalaCreada)
}
```

---

## 🌐 Frontend: Dioxus (REST)

```rust
use dioxus::prelude::*;
use salas_validation::{ValidarSala, validar_nombre, validar_capacidad};

fn app() -> Element {
    let nombre = use_signal(|| String::new());
    let capacidad = use_signal(|| String::new());
    let error = use_signal(|| String::new());

    let crear_sala = move |_| {
        let cap = capacidad().parse::<u32>().unwrap_or(0);

        // ✅ Opción 1: Validar campos individualmente
        if let Err(e) = validar_nombre(&nombre()) {
            error.set(e.mensaje_usuario());
            return;
        }

        if let Err(e) = validar_capacidad(cap) {
            error.set(e.mensaje_usuario());
            return;
        }

        // ✅ Opción 2: Crear request y validar todo junto
        let request = CrearSalaRequest {
            nombre: nombre(),
            capacidad: cap,
        };

        if let Err(e) = request.validar() {
            error.set(e.mensaje_usuario());
            return;
        }

        // Request válido, enviar al backend REST
        spawn(async move {
            let response = reqwest::post("http://localhost:3000/api/salas")
                .json(&request)
                .send()
                .await
                .unwrap();

            // ...
        });
    };

    rsx! {
        div {
            input { oninput: move |e| nombre.set(e.value()) }
            input { oninput: move |e| capacidad.set(e.value()) }
            button { onclick: crear_sala, "Crear Sala" }
            if !error().is_empty() {
                p { style: "color: red", "{error()}" }
            }
        }
    }
}
```

---

## 🖥️ Backend: Servidor gRPC (Tonic)

```rust
use tonic::{Request, Response, Status};
use salas_grpc::proto::{CrearSalaRequest, SalaResponse};
use salas_validation::ValidarSala;

#[tonic::async_trait]
impl SalaService for SalaServiceImpl {
    async fn crear_sala(
        &self,
        request: Request<CrearSalaRequest>,
    ) -> Result<Response<SalaResponse>, Status> {
        let req = request.into_inner();

        // ✅ Validar request (mismas reglas que el frontend)
        req.validar()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        // Lógica de negocio
        let sala = self.repository
            .crear_sala(&req.nombre, req.capacidad)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(sala.into()))
    }
}
```

---

## 🌐 Backend: API REST (Axum)

### Opción 1: Validación manual

```rust
use axum::{Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use salas_grpc::proto::CrearSalaRequest;
use salas_validation::ValidarSala;

#[derive(Deserialize)]
struct CrearSalaDto {
    nombre: String,
    capacidad: u32,
}

async fn crear_sala(
    Json(dto): Json<CrearSalaDto>,
) -> Result<Json<SalaDto>, (StatusCode, String)> {
    // Convertir DTO → Proto
    let request = CrearSalaRequest {
        nombre: dto.nombre,
        capacidad: dto.capacidad,
    };

    // ✅ Validar
    request.validar()
        .map_err(|e| (StatusCode::BAD_REQUEST, e.mensaje_usuario()))?;

    // Lógica de negocio...
    let sala = crear_sala_en_db(request).await?;

    Ok(Json(sala))
}
```

### Opción 2: Extractor personalizado (automático)

```rust
use axum::{
    async_trait,
    extract::{FromRequest, Request},
    http::StatusCode,
    Json,
};
use salas_validation::ValidarSala;

/// Extractor que valida automáticamente
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: ValidarSala + serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

        // ✅ Validar automáticamente
        value.validar()
            .map_err(|e| (StatusCode::BAD_REQUEST, e.mensaje_usuario()))?;

        Ok(ValidatedJson(value))
    }
}

// Handler - validación automática
async fn crear_sala(
    ValidatedJson(request): ValidatedJson<CrearSalaRequest>,
) -> Result<Json<SalaDto>, (StatusCode, String)> {
    // ✅ Ya está validado aquí!
    let sala = crear_sala_en_db(request).await?;
    Ok(Json(sala))
}
```

---

## 🖥️ CLI: Validación de argumentos

```rust
use clap::Parser;
use salas_validation::{validar_nombre, validar_capacidad};

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    nombre: String,

    #[arg(short, long)]
    capacidad: u32,
}

fn main() {
    let args = Args::parse();

    // ✅ Validar argumentos CLI
    if let Err(e) = validar_nombre(&args.nombre) {
        eprintln!("Error: {}", e.mensaje_usuario());
        std::process::exit(1);
    }

    if let Err(e) = validar_capacidad(args.capacidad) {
        eprintln!("Error: {}", e.mensaje_usuario());
        std::process::exit(1);
    }

    println!("✅ Creando sala '{}' con capacidad {}", args.nombre, args.capacidad);
    // ...
}
```

---

## 📱 TUI: Validación en interfaz terminal

```rust
use ratatui::{prelude::*, widgets::*};
use salas_validation::{validar_nombre, NOMBRE_MIN_LENGTH, NOMBRE_MAX_LENGTH};

struct App {
    nombre_input: String,
    error_message: String,
}

impl App {
    fn validar_input(&mut self) {
        match validar_nombre(&self.nombre_input) {
            Ok(()) => {
                self.error_message.clear();
            }
            Err(e) => {
                self.error_message = e.mensaje_usuario();
            }
        }
    }

    fn render(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Length(3)])
            .split(f.area());

        // Input
        let input = Paragraph::new(self.nombre_input.as_str())
            .block(Block::default().borders(Borders::ALL).title(
                format!("Nombre ({}-{} chars)", NOMBRE_MIN_LENGTH, NOMBRE_MAX_LENGTH)
            ));
        f.render_widget(input, chunks[0]);

        // Error message
        if !self.error_message.is_empty() {
            let error = Paragraph::new(self.error_message.as_str())
                .style(Style::default().fg(Color::Red));
            f.render_widget(error, chunks[1]);
        }
    }
}
```

---

## 🧪 Testing: Validar en tests

```rust
use salas_grpc::proto::CrearSalaRequest;
use salas_validation::ValidarSala;

#[test]
fn test_request_valido() {
    let request = CrearSalaRequest {
        nombre: "Sala 101".to_string(),
        capacidad: 50,
    };

    assert!(request.validar().is_ok());
}

#[test]
fn test_request_nombre_muy_corto() {
    let request = CrearSalaRequest {
        nombre: "AB".to_string(),
        capacidad: 50,
    };

    assert!(request.validar().is_err());
}

#[test]
fn test_request_capacidad_cero() {
    let request = CrearSalaRequest {
        nombre: "Sala 101".to_string(),
        capacidad: 0,
    };

    let result = request.validar();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().mensaje_usuario(),
        "La capacidad debe ser al menos 1 persona"
    );
}
```

---

## 🎨 UI Hints: Usar constantes para placeholders

```rust
use iced::widget::text_input;
use salas_validation::{NOMBRE_MIN_LENGTH, NOMBRE_MAX_LENGTH, CAPACIDAD_MIN, CAPACIDAD_MAX};

// En tu función view()
text_input(
    &format!("Nombre ({}-{} caracteres)", NOMBRE_MIN_LENGTH, NOMBRE_MAX_LENGTH),
    &self.nuevo_nombre
)

text_input(
    &format!("Capacidad ({}-{} personas)", CAPACIDAD_MIN, CAPACIDAD_MAX),
    &self.nueva_capacidad
)
```

---

## 🔄 Conversión entre DTOs

Si tienes DTOs REST y quieres validarlos:

```rust
use serde::{Deserialize, Serialize};
use salas_grpc::proto::CrearSalaRequest;
use salas_validation::ValidarSala;

#[derive(Deserialize)]
struct CrearSalaRestDto {
    pub nombre: String,
    pub capacidad: u32,
}

// Conversión automática
impl From<CrearSalaRestDto> for CrearSalaRequest {
    fn from(dto: CrearSalaRestDto) -> Self {
        Self {
            nombre: dto.nombre,
            capacidad: dto.capacidad,
        }
    }
}

// Ahora puedes validar DTOs REST
fn validar_dto_rest(dto: CrearSalaRestDto) -> Result<(), String> {
    let request: CrearSalaRequest = dto.into();
    request.validar()
        .map_err(|e| e.mensaje_usuario())
}
```

---

## 🌟 Ejemplo completo: Flujo end-to-end

```rust
// Frontend (Iced)
Message::CrearSala => {
    let request = CrearSalaRequest {
        nombre: self.nuevo_nombre.clone(),
        capacidad: self.nueva_capacidad.parse().unwrap_or(0),
    };

    // ✅ Validación client-side
    if let Err(e) = request.validar() {
        self.mensaje = format!("❌ {}", e.mensaje_usuario());
        return Task::none();
    }

    // Enviar al backend
    Task::perform(grpc_crear_sala(request), Message::SalaCreada)
}

// Backend (gRPC)
async fn crear_sala(
    &self,
    request: Request<CrearSalaRequest>,
) -> Result<Response<SalaResponse>, Status> {
    let req = request.into_inner();

    // ✅ Validación server-side (mismas reglas)
    req.validar()
        .map_err(|e| Status::invalid_argument(e.to_string()))?;

    // Lógica de negocio...
    let sala = self.repository.crear_sala(req).await?;
    Ok(Response::new(sala))
}
```

**Resultado:** Validaciones consistentes en todo el stack ✅

---

## 📊 Resumen de casos de uso

| Contexto | Método | Código |
|----------|--------|--------|
| **Frontend gRPC** | `request.validar()` | `if let Err(e) = request.validar()` |
| **Frontend REST** | `validar_*()` directas | `validar_nombre(&nombre)?` |
| **Backend gRPC** | `request.validar()` | `.map_err(\|e\| Status::invalid_argument(...))` |
| **Backend REST** | Conversión + `validar()` | `let req: Proto = dto.into(); req.validar()?` |
| **CLI** | Funciones individuales | `validar_nombre(&args.nombre)?` |
| **Tests** | `request.validar()` | `assert!(request.validar().is_ok())` |
| **UI Hints** | Constantes | `NOMBRE_MIN_LENGTH, CAPACIDAD_MAX` |

---

## 💡 Tips

1. **Frontend:** Valida ANTES de hacer la request (mejor UX)
2. **Backend:** SIEMPRE valida (seguridad)
3. **Tests:** Usa `mensaje_usuario()` para verificar mensajes
4. **UI:** Usa las constantes para hints dinámicos
5. **REST:** Convierte DTOs → Proto para reutilizar validaciones

¡Todo centralizado y type-safe! 🎉
