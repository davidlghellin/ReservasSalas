# Salas Validation

Crate de **validaciones compartidas** para las operaciones de salas en el sistema de Reservas.

## 🎯 Propósito

Este crate permite compartir las **mismas reglas de validación** entre:
- ✅ **Frontend** (Iced, Dioxus, Slint, Tauri)
- ✅ **Backend** (servidor gRPC)

De esta forma, las validaciones están **centralizadas** y son consistentes en toda la aplicación.

## 📦 Instalación

Añade la dependencia en tu `Cargo.toml`:

```toml
[dependencies]
salas-validation = { path = "../features/salas/validation" }
```

## 🚀 Uso

### Ejemplo básico

```rust
use salas_validation::{ValidarSala, SalaValidationError};
use salas_grpc::proto::CrearSalaRequest;

let request = CrearSalaRequest {
    nombre: "Sala 101".to_string(),
    capacidad: 50,
};

// Validar el request
match request.validar() {
    Ok(()) => {
        // ✅ Request válido, enviar al backend
        println!("Request válido, creando sala...");
    }
    Err(e) => {
        // ❌ Error de validación
        eprintln!("Error: {}", e.mensaje_usuario());
    }
}
```

### Uso en Iced (frontend)

```rust
use salas_validation::ValidarSala;

Message::CrearSala => {
    let capacidad = self.nueva_capacidad.parse::<u32>().unwrap_or(0);

    let request = CrearSalaRequest {
        nombre: self.nuevo_nombre.clone(),
        capacidad,
    };

    // ✅ Validar ANTES de enviar al backend
    if let Err(e) = request.validar() {
        self.mensaje = format!("❌ {}", e.mensaje_usuario());
        return Task::none();
    }

    // Si pasa validación, hacer la llamada gRPC
    Task::perform(crear_sala(request), Message::SalaCreada)
}
```

### Uso en el backend gRPC

```rust
use salas_validation::ValidarSala;
use tonic::{Request, Response, Status};

async fn crear_sala(
    &self,
    request: Request<CrearSalaRequest>,
) -> Result<Response<SalaResponse>, Status> {
    let req = request.into_inner();

    // ✅ Validar con las mismas reglas del frontend
    req.validar()
        .map_err(|e| Status::invalid_argument(e.to_string()))?;

    // Continuar con la lógica de negocio...
}
```

## 📋 Reglas de validación

### CrearSalaRequest

| Campo | Reglas |
|-------|--------|
| **nombre** | • No vacío<br>• Entre 3 y 100 caracteres<br>• Solo letras, números y espacios |
| **capacidad** | • Mayor que 0<br>• Entre 1 y 500 personas |

### ObtenerSalaRequest / ActivarSalaRequest / DesactivarSalaRequest

| Campo | Reglas |
|-------|--------|
| **id** | • No vacío<br>• Formato UUID válido |

## 🔧 Constantes públicas

Puedes usar las constantes de validación para mostrar hints en la UI:

```rust
use salas_validation::{
    NOMBRE_MIN_LENGTH,
    NOMBRE_MAX_LENGTH,
    CAPACIDAD_MIN,
    CAPACIDAD_MAX,
};

// Ejemplo: mostrar placeholder dinámico
text_input(
    &format!("Nombre ({}-{} caracteres)", NOMBRE_MIN_LENGTH, NOMBRE_MAX_LENGTH),
    &self.nuevo_nombre
)
```

## 🧪 Tests

El crate incluye tests exhaustivos:

```bash
cargo test --manifest-path crates/features/salas/validation/Cargo.toml
```

**Tests incluidos:**
- ✅ Validación de nombres (vacío, muy corto, muy largo, caracteres inválidos)
- ✅ Validación de capacidad (cero, fuera de rango, válida)
- ✅ Validación de IDs (vacío, formato inválido, UUID válido)
- ✅ Validación de requests completos

## 🎨 Mensajes de error amigables

El trait `SalaValidationError` proporciona mensajes para el usuario:

```rust
match request.validar() {
    Ok(()) => { /* ... */ }
    Err(e) => {
        // Mensaje técnico (para logs)
        eprintln!("Error técnico: {}", e);

        // Mensaje amigable (para UI)
        self.mensaje = format!("❌ {}", e.mensaje_usuario());
    }
}
```

**Ejemplos de mensajes:**

| Error | Mensaje técnico | Mensaje usuario |
|-------|----------------|-----------------|
| `NombreVacio` | "El nombre no puede estar vacío" | "Por favor, ingresa un nombre para la sala" |
| `CapacidadCero` | "La capacidad debe ser mayor que 0" | "La capacidad debe ser al menos 1 persona" |
| `IdFormatoInvalido` | "El ID debe ser un UUID válido" | "El ID de la sala no es válido" |

## 🏗️ Arquitectura

```
crates/features/salas/validation/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs        # Exports públicos
    ├── error.rs      # Definición de errores
    └── sala.rs       # Trait ValidarSala + implementaciones
```

## 🔄 Extensión

Para añadir nuevas validaciones:

1. Añade el error en `error.rs`:
```rust
#[error("Nueva regla de validación")]
NuevaRegla,
```

2. Implementa la validación en `sala.rs`:
```rust
fn validar_nueva_regla(valor: &str) -> Result<(), SalaValidationError> {
    // Lógica de validación
    Ok(())
}
```

3. Actualiza el trait para el request correspondiente:
```rust
impl ValidarSala for MiRequest {
    fn validar(&self) -> Result<(), SalaValidationError> {
        validar_nueva_regla(&self.campo)?;
        Ok(())
    }
}
```

## 📊 Ventajas

| Ventaja | Descripción |
|---------|-------------|
| ✅ **DRY** | Una sola implementación para frontend y backend |
| ✅ **Type-safe** | Todo en Rust, sin runtime errors |
| ✅ **Testeable** | Tests unitarios independientes |
| ✅ **Mensajes consistentes** | Mismo UX en toda la app |
| ✅ **Fácil mantenimiento** | Un solo lugar para actualizar reglas |

## 🔗 Ver también

- [salas-grpc](../grpc/) - Definiciones Protocol Buffers
- [app-desktop-iced](../../../app-desktop-iced/) - Ejemplo de uso en frontend
