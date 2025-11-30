# Salas Infrastructure - Adaptadores de Persistencia

Capa de infraestructura que implementa los **adaptadores** (adapters) para persistir salas según la arquitectura hexagonal.

## 🏗️ Arquitectura Hexagonal

```
┌─────────────────────────────────────────┐
│           Domain (Core)                  │
│  • Sala (entity)                         │
│  • SalaError                             │
│  • Reglas de negocio                     │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│        Application (Ports)               │
│  • SalaService (use cases)               │
│  • SalaRepository trait ← PORT           │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│     Infrastructure (Adapters)            │
│  • memory_repository.rs  ← InMemory      │
│  • file_repository.rs    ← JSON File     │
│  • postgres_repository.rs (futuro)       │
└──────────────────────────────────────────┘
```

## 📦 Adaptadores Disponibles

### 1. `InMemorySalaRepository`

Repositorio **en memoria** usando `HashMap`. Ideal para:
- ✅ Tests
- ✅ Desarrollo rápido
- ✅ Demos
- ❌ Producción (los datos se pierden al reiniciar)

**Uso:**
```rust
use salas_infrastructure::InMemorySalaRepository;
use salas_application::SalaRepository;

let repo = InMemorySalaRepository::new();

// Usar directamente
let sala = Sala::new("123".to_string(), "Sala 1".to_string(), 50)?;
repo.guardar(&sala).await?;
```

---

### 2. `FileSalaRepository` ⭐ Nuevo

Repositorio que **persiste en archivo JSON**. Ideal para:
- ✅ Aplicaciones de escritorio (Iced, Tauri, etc.)
- ✅ CLIs que necesitan persistencia
- ✅ Prototipos
- ✅ Apps pequeñas sin necesidad de DB

**Características:**
- 💾 **Persistencia** - Los datos sobreviven a reinicios
- 📝 **Formato JSON** - Legible y editable manualmente
- ⚡ **Cache en memoria** - Lecturas rápidas
- 🔒 **Thread-safe** - Usa `tokio::sync::RwLock`
- 📁 **Auto-create directorio** - Crea carpetas si no existen

**Uso básico:**
```rust
use salas_infrastructure::FileSalaRepository;
use salas_application::SalaRepository;
use std::path::PathBuf;

// Opción 1: Ruta personalizada
let repo = FileSalaRepository::new(PathBuf::from("./mi_app/salas.json"));

// Opción 2: Ruta por defecto (./data/salas.json)
let repo = FileSalaRepository::default_path();

// ✅ IMPORTANTE: Inicializar para cargar datos existentes
repo.init().await?;

// Ahora puedes usar el repositorio
let sala = Sala::new("123".to_string(), "Sala 1".to_string(), 50)?;
repo.guardar(&sala).await?;  // Guarda en memoria Y en archivo

// Listar salas
let salas = repo.listar().await?;
```

**Formato del archivo JSON:**
```json
{
  "salas": {
    "123": {
      "id": "123",
      "nombre": "Sala de Conferencias",
      "capacidad": 50,
      "activa": true
    },
    "456": {
      "id": "456",
      "nombre": "Sala de Reuniones",
      "capacidad": 20,
      "activa": false
    }
  }
}
```

---

## 🚀 Ejemplo: Usar en Iced

```rust
use iced::Task;
use salas_infrastructure::FileSalaRepository;
use salas_application::{SalaRepository, SalaServiceImpl};
use std::sync::Arc;
use std::path::PathBuf;

// En el init de tu app
async fn init_app() -> App {
    // 1. Crear repositorio de archivo
    let repo = FileSalaRepository::new(PathBuf::from("./data/salas.json"));

    // 2. Cargar datos existentes
    repo.init().await.unwrap();

    // 3. Crear servicio con el repositorio
    let service = Arc::new(SalaServiceImpl::new(Arc::new(repo)));

    App {
        service,
        // ...
    }
}

// Los datos ahora persisten entre sesiones ✅
```

---

## 🔄 Cambiar de Adaptador

El código de aplicación NO cambia. Solo cambias el adaptador:

```rust
// Opción A: En memoria (testing)
let repo: Arc<dyn SalaRepository> = Arc::new(InMemorySalaRepository::new());

// Opción B: Archivo JSON (desktop apps)
let repo: Arc<dyn SalaRepository> = {
    let file_repo = FileSalaRepository::default_path();
    file_repo.init().await?;
    Arc::new(file_repo)
};

// Opción C: PostgreSQL (futuro - producción)
// let repo: Arc<dyn SalaRepository> = Arc::new(PostgresRepository::new(pool));

// El servicio funciona con cualquiera
let service = SalaServiceImpl::new(repo);
```

---

## 🧪 Testing

```bash
# Tests del crate
cargo test --manifest-path crates/features/salas/infrastructure/Cargo.toml

# Tests incluidos:
# ✅ test_guardar_y_obtener_sala
# ✅ test_listar_salas
# ✅ test_actualizar_sala
# ✅ test_persistencia_en_archivo
# ✅ test_archivo_json_formato_correcto
```

---

## 📊 Comparación de Adaptadores

| Característica | InMemory | File (JSON) | PostgreSQL (futuro) |
|----------------|----------|-------------|---------------------|
| **Persistencia** | ❌ | ✅ | ✅ |
| **Velocidad** | ⚡⚡⚡ | ⚡⚡ | ⚡ |
| **Setup** | Cero | Mínimo | Medio |
| **Ideal para** | Tests | Desktop/CLI | Producción |
| **Escalabilidad** | Baja | Media | Alta |
| **Transacciones** | N/A | No | ✅ |
| **Concurrent writes** | ⚠️ | ⚠️ | ✅ |

---

## 🔐 Thread Safety

Ambos adaptadores son **thread-safe**:

- `InMemorySalaRepository`: `Arc<RwLock<HashMap>>`
- `FileSalaRepository`: `Arc<RwLock<HashMap>>` + async I/O

Puedes clonar y compartir entre threads/tasks:

```rust
let repo = Arc::new(FileSalaRepository::default_path());
repo.init().await?;

let repo_clone = repo.clone();

tokio::spawn(async move {
    repo_clone.listar().await.unwrap();
});
```

---

## 🎯 Próximos Adaptadores

### PostgresRepository (Futuro)

```rust
pub struct PostgresSalaRepository {
    pool: PgPool,
}

impl PostgresSalaRepository {
    pub async fn new(database_url: &str) -> Result<Self, SalaError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl SalaRepository for PostgresSalaRepository {
    async fn guardar(&self, sala: &Sala) -> Result<(), SalaError> {
        sqlx::query!(
            "INSERT INTO salas (id, nombre, capacidad, activa) VALUES ($1, $2, $3, $4)",
            sala.id,
            sala.nombre,
            sala.capacidad as i32,
            sala.activa
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
    // ...
}
```

---

## 💡 Tips

### 1. Inicialización

**Siempre** llama a `.init()` en `FileSalaRepository`:

```rust
let repo = FileSalaRepository::new(path);
repo.init().await?;  // ← No olvides esto
```

### 2. Manejo de errores

```rust
match repo.guardar(&sala).await {
    Ok(()) => println!("✅ Sala guardada"),
    Err(SalaError::ErrorRepositorio(msg)) => {
        eprintln!("❌ Error de persistencia: {}", msg);
    }
    Err(e) => eprintln!("❌ Error: {:?}", e),
}
```

### 3. Configurar ruta desde env

```rust
use std::env;

let data_dir = env::var("SALAS_DATA_DIR")
    .unwrap_or_else(|_| "./data".to_string());

let repo = FileSalaRepository::new(
    PathBuf::from(data_dir).join("salas.json")
);
```

---

## 📚 Ver también

- [Domain](../domain/) - Entidades y reglas de negocio
- [Application](../application/) - Casos de uso y ports
- [API](../api/) - REST API (HTTP)
- [gRPC](../grpc/) - gRPC server
