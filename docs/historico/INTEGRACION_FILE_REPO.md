# Integración de FileSalaRepository en Iced

Dos formas de integrar el adaptador de fichero en tu app Iced actual.

## 🎯 Opción 1: Reemplazar gRPC (App standalone)

Usar `FileSalaRepository` directamente sin necesidad de servidor gRPC.

### 1. Añadir dependencias

```toml
# crates/app-desktop-iced/Cargo.toml
[dependencies]
iced = { version = "0.13", features = ["tokio"] }
tokio = { workspace = true, features = ["full"] }

# Capas de dominio
salas-domain = { path = "../features/salas/domain" }
salas-application = { path = "../features/salas/application" }
salas-infrastructure = { path = "../features/salas/infrastructure" }
salas-validation = { path = "../features/salas/validation" }

# Notificaciones
notify-rust = "4"
```

### 2. Modificar main.rs

```rust
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length, Task, Theme};
use notify_rust::Notification;
use std::path::PathBuf;
use std::sync::Arc;

// Imports de tu dominio
use salas_domain::Sala;
use salas_application::SalaService;
use salas_infrastructure::FileSalaRepository;
use salas_validation::ValidarSala;

fn main() -> iced::Result {
    iced::application("Gestión de Salas - Iced (File)", App::update, App::view)
        .theme(App::theme)
        .run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {
    // Inicialización
    RepositoryInitialized(Result<Vec<Sala>, String>),

    // CRUD operations
    SalasCargadas(Result<Vec<Sala>, String>),
    SalaCreada(Result<Sala, String>),
    SalaActivada(Result<Sala, String>),
    SalaDesactivada(Result<Sala, String>),

    // UI events
    NombreChanged(String),
    CapacidadChanged(String),
    CrearSala,
    ActivarSala(String),
    DesactivarSala(String),
    ActualizarSalas,
}

struct App {
    // Servicio que usa el repositorio
    service: Option<Arc<dyn SalaService>>,

    // Estado UI
    salas: Vec<Sala>,
    nuevo_nombre: String,
    nueva_capacidad: String,
    mensaje: String,
    loading: bool,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let app = Self {
            service: None,
            salas: Vec::new(),
            nuevo_nombre: String::new(),
            nueva_capacidad: String::from("10"),
            mensaje: String::new(),
            loading: true,
        };

        // Inicializar repositorio y cargar salas
        let task = Task::perform(
            inicializar_repositorio(),
            Message::RepositoryInitialized
        );

        (app, task)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::RepositoryInitialized(Ok(salas)) => {
                // Guardar referencia al servicio
                // (Ya está inicializado en la función async)
                self.salas = salas;
                self.loading = false;
                self.mensaje = format!("✅ {} salas cargadas", self.salas.len());
                Task::none()
            }

            Message::RepositoryInitialized(Err(e)) => {
                self.loading = false;
                self.mensaje = format!("❌ Error: {}", e);
                Task::none()
            }

            Message::CrearSala => {
                let capacidad = match self.nueva_capacidad.parse::<u32>() {
                    Ok(c) => c,
                    Err(_) => {
                        self.mensaje = "❌ Capacidad inválida".to_string();
                        return Task::none();
                    }
                };

                let request = salas_grpc::proto::CrearSalaRequest {
                    nombre: self.nuevo_nombre.clone(),
                    capacidad,
                };

                // Validar ANTES de crear
                if let Err(e) = request.validar() {
                    self.mensaje = format!("❌ {}", e.mensaje_usuario());
                    return Task::none();
                }

                self.loading = true;
                let nombre = self.nuevo_nombre.clone();

                Task::perform(
                    crear_sala(nombre, capacidad),
                    Message::SalaCreada
                )
            }

            Message::SalaCreada(Ok(sala)) => {
                self.salas.push(sala.clone());
                self.mensaje = format!("✅ Sala '{}' guardada en disco", sala.nombre());

                mostrar_notificacion(
                    "✅ Sala creada",
                    &format!("Sala '{}' guardada", sala.nombre()),
                    TipoNotificacion::Exito,
                );

                self.nuevo_nombre.clear();
                self.nueva_capacidad = String::from("10");
                self.loading = false;
                Task::none()
            }

            Message::SalaCreada(Err(e)) => {
                self.mensaje = format!("❌ {}", e);
                self.loading = false;
                Task::none()
            }

            Message::ActivarSala(id) => {
                self.loading = true;
                Task::perform(
                    activar_sala(id),
                    Message::SalaActivada
                )
            }

            Message::SalaActivada(Ok(sala)) => {
                // Actualizar en la lista local
                if let Some(s) = self.salas.iter_mut().find(|s| s.id() == sala.id()) {
                    *s = sala.clone();
                }

                self.mensaje = format!("✅ Sala '{}' activada", sala.nombre());
                self.loading = false;
                Task::none()
            }

            Message::DesactivarSala(id) => {
                self.loading = true;
                Task::perform(
                    desactivar_sala(id),
                    Message::SalaDesactivada
                )
            }

            Message::SalaDesactivada(Ok(sala)) => {
                if let Some(s) = self.salas.iter_mut().find(|s| s.id() == sala.id()) {
                    *s = sala.clone();
                }

                self.mensaje = format!("✅ Sala '{}' desactivada", sala.nombre());
                self.loading = false;
                Task::none()
            }

            Message::ActualizarSalas => {
                self.loading = true;
                Task::perform(
                    listar_salas(),
                    Message::SalasCargadas
                )
            }

            Message::SalasCargadas(Ok(salas)) => {
                self.salas = salas;
                self.loading = false;
                Task::none()
            }

            _ => Task::none()
        }
    }

    // ... view() igual que antes
}

// -------- Funciones async con el repositorio --------

use once_cell::sync::Lazy;
use tokio::sync::RwLock;

// Repositorio global (singleton)
static REPOSITORY: Lazy<Arc<RwLock<Option<Arc<FileSalaRepository>>>>> =
    Lazy::new(|| Arc::new(RwLock::new(None)));

async fn inicializar_repositorio() -> Result<Vec<Sala>, String> {
    // Crear repositorio
    let repo = Arc::new(FileSalaRepository::new(
        PathBuf::from("./data/salas.json")
    ));

    // Inicializar (cargar desde archivo)
    repo.init().await
        .map_err(|e| format!("Error al inicializar: {}", e))?;

    // Guardar en global
    {
        let mut repo_guard = REPOSITORY.write().await;
        *repo_guard = Some(repo.clone());
    }

    // Cargar salas
    repo.listar().await
        .map_err(|e| format!("{}", e))
}

async fn crear_sala(nombre: String, capacidad: u32) -> Result<Sala, String> {
    let repo = REPOSITORY.read().await;
    let repo = repo.as_ref()
        .ok_or("Repositorio no inicializado")?;

    // Crear sala usando el dominio
    let id = uuid::Uuid::new_v4().to_string();
    let sala = Sala::new(id, nombre, capacidad)
        .map_err(|e| format!("{:?}", e))?;

    // Guardar
    repo.guardar(&sala).await
        .map_err(|e| format!("{}", e))?;

    Ok(sala)
}

async fn activar_sala(id: String) -> Result<Sala, String> {
    let repo = REPOSITORY.read().await;
    let repo = repo.as_ref()
        .ok_or("Repositorio no inicializado")?;

    // Obtener sala
    let mut sala = repo.obtener(&id).await
        .map_err(|e| format!("{}", e))?
        .ok_or("Sala no encontrada")?;

    // Activar
    sala.activar();

    // Actualizar
    repo.actualizar(&sala).await
        .map_err(|e| format!("{}", e))?;

    Ok(sala)
}

async fn desactivar_sala(id: String) -> Result<Sala, String> {
    let repo = REPOSITORY.read().await;
    let repo = repo.as_ref()
        .ok_or("Repositorio no inicializado")?;

    let mut sala = repo.obtener(&id).await
        .map_err(|e| format!("{}", e))?
        .ok_or("Sala no encontrada")?;

    sala.desactivar();

    repo.actualizar(&sala).await
        .map_err(|e| format!("{}", e))?;

    Ok(sala)
}

async fn listar_salas() -> Result<Vec<Sala>, String> {
    let repo = REPOSITORY.read().await;
    let repo = repo.as_ref()
        .ok_or("Repositorio no inicializado")?;

    repo.listar().await
        .map_err(|e| format!("{}", e))
}

// Notificaciones y tema igual que antes...
```

---

## 🎯 Opción 2: Híbrido (gRPC + File)

Mantener gRPC pero que el **servidor** use `FileSalaRepository`.

### Servidor gRPC con FileSalaRepository

```rust
// En tu servidor gRPC
use salas_grpc::proto::sala_service_server::{SalaService, SalaServiceServer};
use salas_infrastructure::FileSalaRepository;
use salas_application::{SalaRepository, SalaServiceImpl};
use std::sync::Arc;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Crear repositorio de archivo
    let repo = Arc::new(FileSalaRepository::new(
        PathBuf::from("./data/salas.json")
    ));

    // Inicializar
    repo.init().await?;

    // Crear servicio con el repositorio
    let service = SalaServiceImpl::new(repo);

    // Crear servidor gRPC
    let addr = "0.0.0.0:50051".parse()?;

    Server::builder()
        .add_service(SalaServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
```

Ahora tu app Iced usa gRPC, pero **el servidor persiste en JSON** ✅

---

## 📊 Comparación

| Aspecto | Opción 1: File directo | Opción 2: gRPC + File |
|---------|------------------------|------------------------|
| **Setup** | Simple | Medio |
| **Dependencias** | Sin gRPC/tonic | Con gRPC |
| **Servidor** | No necesita | Sí necesita |
| **Persistencia** | JSON local | JSON en servidor |
| **Ideal para** | App desktop standalone | Cliente-servidor |
| **Complejidad** | Baja | Media |

---

## 💡 Recomendación

**Para app desktop:** Usa **Opción 1** (File directo)
- Más simple
- No necesitas servidor corriendo
- Datos locales en la máquina del usuario

**Para arquitectura cliente-servidor:** Usa **Opción 2** (gRPC + File en servidor)
- Múltiples clientes
- Datos centralizados
- Más escalable

---

## 🚀 Próximo paso

¿Qué prefieres?
1. **Opción 1**: Te ayudo a modificar tu `main.rs` actual para usar File directo
2. **Opción 2**: Te ayudo a crear el servidor gRPC con FileSalaRepository
