# Ejemplo: Usar FileSalaRepository en Iced

Este ejemplo muestra cómo integrar el adaptador de fichero JSON en una aplicación Iced.

## 🎯 Objetivo

Que las salas creadas en Iced **persistan entre sesiones** (se guarden en disco).

## 📝 Implementación

### 1. Añadir dependencia

```toml
# crates/app-desktop-iced/Cargo.toml
[dependencies]
salas-infrastructure = { path = "../features/salas/infrastructure" }
salas-application = { path = "../features/salas/application" }
```

### 2. Inicializar repositorio en el main

```rust
use iced::{Application, Settings};
use salas_infrastructure::FileSalaRepository;
use std::path::PathBuf;

fn main() -> iced::Result {
    // Configurar ruta del archivo
    let data_path = PathBuf::from("./data/salas.json");

    App::run(Settings {
        flags: data_path,
        ..Default::default()
    })
}
```

### 3. Cargar repositorio en `new()`

```rust
use salas_infrastructure::FileSalaRepository;
use salas_application::{SalaRepository, SalaServiceImpl};
use std::sync::Arc;

struct App {
    service: Arc<SalaServiceImpl<FileSalaRepository>>,
    salas: Vec<Sala>,
    loading: bool,
    // ...
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = PathBuf;

    fn new(data_path: PathBuf) -> (Self, Task<Message>) {
        let app = Self {
            service: Arc::new(SalaServiceImpl::new(Arc::new(
                FileSalaRepository::new(data_path)
            ))),
            salas: Vec::new(),
            loading: true,
            // ...
        };

        // Inicializar y cargar salas
        let service = app.service.clone();
        let task = Task::perform(
            async move {
                // Inicializar repositorio (cargar desde archivo)
                service.repository.init().await.ok();

                // Listar salas existentes
                service.listar_salas().await
            },
            Message::SalasCargadas
        );

        (app, task)
    }

    // ...
}
```

### 4. Flujo completo

```rust
#[derive(Debug, Clone)]
enum Message {
    // Inicialización
    SalasCargadas(Result<Vec<Sala>, String>),

    // Crear sala
    CrearSala,
    SalaCreada(Result<Sala, String>),

    // ...
}

impl Application for App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SalasCargadas(Ok(salas)) => {
                self.salas = salas;
                self.loading = false;
                Task::none()
            }

            Message::CrearSala => {
                let service = self.service.clone();
                let nombre = self.nuevo_nombre.clone();
                let capacidad = self.nueva_capacidad;

                Task::perform(
                    async move {
                        service.crear_sala(nombre, capacidad).await
                    },
                    Message::SalaCreada
                )
            }

            Message::SalaCreada(Ok(sala)) => {
                self.salas.push(sala);
                self.mensaje = "✅ Sala creada y guardada en archivo".to_string();

                // ✅ Los datos ya están persistidos en disco
                Task::none()
            }

            // ...
        }
    }
}
```

## 🎨 Ejemplo completo

```rust
use iced::{widget::*, Alignment, Element, Length, Task, Theme};
use salas_domain::Sala;
use salas_infrastructure::FileSalaRepository;
use salas_application::{SalaRepository, SalaServiceImpl};
use std::path::PathBuf;
use std::sync::Arc;

fn main() -> iced::Result {
    App::run(Settings {
        flags: PathBuf::from("./data/salas.json"),
        ..Default::default()
    })
}

struct App {
    service: Arc<SalaServiceImpl<FileSalaRepository>>,
    salas: Vec<Sala>,
    nuevo_nombre: String,
    nueva_capacidad: String,
    mensaje: String,
    loading: bool,
}

#[derive(Debug, Clone)]
enum Message {
    SalasCargadas(Result<Vec<Sala>, String>),
    NombreChanged(String),
    CapacidadChanged(String),
    CrearSala,
    SalaCreada(Result<Sala, String>),
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = PathBuf;

    fn new(data_path: PathBuf) -> (Self, Task<Message>) {
        let service = Arc::new(SalaServiceImpl::new(Arc::new(
            FileSalaRepository::new(data_path)
        )));

        let app = Self {
            service: service.clone(),
            salas: Vec::new(),
            nuevo_nombre: String::new(),
            nueva_capacidad: String::new(),
            mensaje: String::new(),
            loading: true,
        };

        // Cargar salas existentes
        let task = Task::perform(
            async move {
                // Inicializar repositorio
                if let Err(e) = service.repository.init().await {
                    return Err(format!("Error al inicializar: {}", e));
                }

                // Listar salas
                service.listar_salas().await
                    .map_err(|e| format!("{}", e))
            },
            Message::SalasCargadas
        );

        (app, task)
    }

    fn title(&self) -> String {
        "Gestor de Salas - Persistente".into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SalasCargadas(Ok(salas)) => {
                self.salas = salas;
                self.loading = false;
                self.mensaje = format!("✅ {} salas cargadas desde archivo", self.salas.len());
                Task::none()
            }

            Message::SalasCargadas(Err(e)) => {
                self.loading = false;
                self.mensaje = format!("❌ Error al cargar: {}", e);
                Task::none()
            }

            Message::NombreChanged(nombre) => {
                self.nuevo_nombre = nombre;
                Task::none()
            }

            Message::CapacidadChanged(capacidad) => {
                self.nueva_capacidad = capacidad;
                Task::none()
            }

            Message::CrearSala => {
                let service = self.service.clone();
                let nombre = self.nuevo_nombre.clone();
                let capacidad = self.nueva_capacidad.parse().unwrap_or(0);

                self.loading = true;

                Task::perform(
                    async move {
                        service.crear_sala(nombre, capacidad).await
                            .map_err(|e| format!("{}", e))
                    },
                    Message::SalaCreada
                )
            }

            Message::SalaCreada(Ok(sala)) => {
                self.salas.push(sala.clone());
                self.mensaje = format!(
                    "✅ Sala '{}' guardada en ./data/salas.json",
                    sala.nombre
                );
                self.nuevo_nombre.clear();
                self.nueva_capacidad.clear();
                self.loading = false;
                Task::none()
            }

            Message::SalaCreada(Err(e)) => {
                self.mensaje = format!("❌ Error: {}", e);
                self.loading = false;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let title = text("Gestor de Salas - JSON File")
            .size(32)
            .width(Length::Fill)
            .horizontal_alignment(alignment::Horizontal::Center);

        // Formulario
        let form = row![
            text_input("Nombre", &self.nuevo_nombre)
                .on_input(Message::NombreChanged)
                .padding(10),
            text_input("Capacidad", &self.nueva_capacidad)
                .on_input(Message::CapacidadChanged)
                .padding(10),
            button("Crear Sala")
                .on_press(Message::CrearSala)
                .padding(10),
        ]
        .spacing(10);

        // Mensaje
        let mensaje = if !self.mensaje.is_empty() {
            container(text(&self.mensaje))
                .padding(10)
                .width(Length::Fill)
        } else {
            container(Space::new(Length::Fill, Length::Shrink))
        };

        // Lista de salas
        let lista: Element<_> = if self.loading {
            text("Cargando...").into()
        } else if self.salas.is_empty() {
            text("No hay salas. ¡Crea la primera!").into()
        } else {
            column(
                self.salas
                    .iter()
                    .map(|sala| {
                        row![
                            text(&sala.nombre).width(Length::FillPortion(3)),
                            text(format!("Cap: {}", sala.capacidad))
                                .width(Length::FillPortion(1)),
                            text(if sala.activa { "✅" } else { "❌" })
                                .width(Length::Shrink),
                        ]
                        .spacing(10)
                        .padding(5)
                        .into()
                    })
                    .collect()
            )
            .spacing(5)
            .into()
        };

        let content = column![
            title,
            Space::new(Length::Fill, 20),
            form,
            mensaje,
            Space::new(Length::Fill, 20),
            text("Salas guardadas:").size(20),
            scrollable(lista),
        ]
        .spacing(10)
        .padding(20)
        .align_items(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::TokyoNight
    }
}
```

## 📁 Estructura del archivo generado

Después de crear algunas salas, el archivo `./data/salas.json` se verá así:

```json
{
  "salas": {
    "f7a3b5c8-1234-5678-90ab-cdef12345678": {
      "id": "f7a3b5c8-1234-5678-90ab-cdef12345678",
      "nombre": "Sala Principal",
      "capacidad": 100,
      "activa": true
    },
    "a2b4c6d8-9876-5432-10fe-dcba98765432": {
      "id": "a2b4c6d8-9876-5432-10fe-dcba98765432",
      "nombre": "Sala de Reuniones",
      "capacidad": 20,
      "activa": true
    }
  }
}
```

## ✨ Resultado

1. **Primera ejecución**: No hay archivo → App empieza vacía
2. **Crear salas**: Se guardan en `./data/salas.json`
3. **Cerrar app**: Los datos quedan en disco
4. **Volver a abrir**: ✅ Las salas se cargan automáticamente

¡Las salas persisten entre sesiones! 🎉

## 🔧 Configuración avanzada

### Ruta configurable

```rust
use std::env;

fn main() -> iced::Result {
    let data_path = env::var("SALAS_DATA_FILE")
        .unwrap_or_else(|_| "./data/salas.json".to_string());

    App::run(Settings {
        flags: PathBuf::from(data_path),
        ..Default::default()
    })
}
```

### Backup automático

```rust
use tokio::fs;

async fn backup_salas() -> Result<(), std::io::Error> {
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_path = format!("./data/backups/salas_{}.json", timestamp);

    fs::copy("./data/salas.json", backup_path).await?;
    Ok(())
}
```

## 💡 Tips

1. **Directorio de datos**: Crea `./data` en .gitignore
2. **Backup**: Haz backups periódicos del JSON
3. **Validación**: El archivo es editable manualmente (útil para debug)
4. **Performance**: El cache en memoria hace lecturas muy rápidas

