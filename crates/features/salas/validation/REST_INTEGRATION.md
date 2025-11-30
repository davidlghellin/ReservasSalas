# Integración con REST API

El crate `salas-validation` también funciona perfectamente con APIs REST. Las validaciones son **independientes del protocolo** (gRPC o REST).

## 🎯 Dos enfoques

### Enfoque 1: Validar DTOs REST directamente

Si tu REST API tiene sus propios structs (DTOs), implementa `ValidarSala` para ellos:

```rust
// En tu crate de API REST
use serde::{Deserialize, Serialize};
use salas_validation::{ValidarSala, SalaValidationError};

#[derive(Debug, Serialize, Deserialize)]
pub struct CrearSalaDto {
    pub nombre: String,
    pub capacidad: u32,
}

// ✅ Implementar el trait para tu DTO REST
impl ValidarSala for CrearSalaDto {
    fn validar(&self) -> Result<(), SalaValidationError> {
        // Reutilizar las funciones de validación del módulo
        salas_validation::validar_nombre(&self.nombre)?;
        salas_validation::validar_capacidad(self.capacidad)?;
        Ok(())
    }
}
```

Pero espera... las funciones `validar_nombre()` y `validar_capacidad()` son privadas.

**Solución:** Exportarlas públicamente.

---

### Enfoque 2: Convertir DTO REST → Proto (Recomendado ✅)

Este es el enfoque **más limpio y DRY**:

```rust
use axum::{Json, http::StatusCode};
use salas_grpc::proto::CrearSalaRequest;
use salas_validation::ValidarSala;

#[derive(Debug, Serialize, Deserialize)]
pub struct CrearSalaDto {
    pub nombre: String,
    pub capacidad: u32,
}

// Conversión de DTO REST → Proto
impl From<CrearSalaDto> for CrearSalaRequest {
    fn from(dto: CrearSalaDto) -> Self {
        CrearSalaRequest {
            nombre: dto.nombre,
            capacidad: dto.capacidad,
        }
    }
}

// Handler REST
async fn crear_sala(
    Json(dto): Json<CrearSalaDto>,
) -> Result<Json<SalaResponse>, (StatusCode, String)> {
    // Convertir DTO → Proto
    let request: CrearSalaRequest = dto.into();

    // ✅ Validar usando el mismo trait que gRPC
    request.validar()
        .map_err(|e| {
            (StatusCode::BAD_REQUEST, e.mensaje_usuario())
        })?;

    // Continuar con lógica de negocio...
    let sala = crear_sala_en_db(request).await?;

    Ok(Json(sala))
}
```

---

## 🏗️ Arquitectura recomendada

```
┌─────────────────────────────────────────────────┐
│           Capa de Presentación                  │
├─────────────────────────────────────────────────┤
│  REST API (DTOs)     │    gRPC API (Proto)      │
│  CrearSalaDto        │    CrearSalaRequest      │
└─────────┬────────────┴────────────┬─────────────┘
          │                         │
          │ convierte a             │ usa directamente
          ▼                         ▼
┌─────────────────────────────────────────────────┐
│         salas-validation (Capa Común)           │
│                                                  │
│  ValidarSala trait                              │
│  • validar_nombre()                             │
│  • validar_capacidad()                          │
│  • validar_id()                                 │
└─────────────────────────────────────────────────┘
```

---

## 📝 Ejemplo completo con Axum

```rust
use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use salas_grpc::proto::{CrearSalaRequest, ActivarSalaRequest};
use salas_validation::ValidarSala;

// DTOs REST
#[derive(Debug, Serialize, Deserialize)]
pub struct CrearSalaDto {
    pub nombre: String,
    pub capacidad: u32,
}

#[derive(Debug, Serialize)]
pub struct SalaDto {
    pub id: String,
    pub nombre: String,
    pub capacidad: u32,
    pub activa: bool,
}

// Conversiones
impl From<CrearSalaDto> for CrearSalaRequest {
    fn from(dto: CrearSalaDto) -> Self {
        Self {
            nombre: dto.nombre,
            capacidad: dto.capacidad,
        }
    }
}

// Handlers
async fn crear_sala(
    Json(dto): Json<CrearSalaDto>,
) -> Result<Json<SalaDto>, (StatusCode, String)> {
    let request: CrearSalaRequest = dto.into();

    // ✅ Validar
    request.validar()
        .map_err(|e| (StatusCode::BAD_REQUEST, e.mensaje_usuario()))?;

    // Lógica de negocio (omitida)
    todo!()
}

async fn activar_sala(
    Path(id): Path<String>,
) -> Result<Json<SalaDto>, (StatusCode, String)> {
    let request = ActivarSalaRequest { id };

    // ✅ Validar ID (UUID)
    request.validar()
        .map_err(|e| (StatusCode::BAD_REQUEST, e.mensaje_usuario()))?;

    // Lógica de negocio (omitida)
    todo!()
}

// Router
pub fn app() -> Router {
    Router::new()
        .route("/api/salas", post(crear_sala))
        .route("/api/salas/:id/activar", post(activar_sala))
}
```

---

## 🔄 Comparación de enfoques

| Aspecto | Enfoque 1: Impl directo | Enfoque 2: Conversión (✅) |
|---------|-------------------------|----------------------------|
| **DRY** | ⚠️ Código duplicado | ✅ Reutiliza todo |
| **Mantenimiento** | ❌ Dos implementaciones | ✅ Una sola |
| **Consistencia** | ⚠️ Puede divergir | ✅ Garantizada |
| **Complejidad** | Media | Baja |

---

## 🎨 Extractor personalizado de Axum (avanzado)

Para automatizar la validación:

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
        // Extraer JSON
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

        // ✅ Validar automáticamente
        value.validar()
            .map_err(|e| (StatusCode::BAD_REQUEST, e.mensaje_usuario()))?;

        Ok(ValidatedJson(value))
    }
}

// Uso en handlers - validación automática
async fn crear_sala(
    ValidatedJson(request): ValidatedJson<CrearSalaRequest>,
) -> Result<Json<SalaDto>, (StatusCode, String)> {
    // ✅ Ya está validado aquí!
    // Continuar con lógica...
    todo!()
}
```

---

## 📊 Ventajas de usar el mismo sistema para REST y gRPC

| Ventaja | Descripción |
|---------|-------------|
| ✅ **DRY** | Una sola implementación para ambos protocolos |
| ✅ **Consistencia** | Mismos errores en REST y gRPC |
| ✅ **Mantenimiento** | Cambiar regla en un solo lugar |
| ✅ **Testing** | Mismos tests para ambas APIs |
| ✅ **Type-safe** | Todo en Rust |

---

## 🚀 Ejemplo de respuesta REST con validación

### Request inválido:
```bash
curl -X POST http://localhost:3000/api/salas \
  -H "Content-Type: application/json" \
  -d '{"nombre": "AB", "capacidad": 50}'
```

### Response:
```json
HTTP/1.1 400 Bad Request
{
  "error": "El nombre debe tener entre 3 y 100 caracteres. Actualmente tiene 2"
}
```

**¡Mismo mensaje que en Iced!** ✅

---

## 🔗 Flujo completo (REST + gRPC)

```
Frontend Iced (gRPC)              Frontend Web (REST)
        ↓                                  ↓
Validación (salas-validation)    Validación (salas-validation)
        ↓                                  ↓
    gRPC API                           REST API
        ↓                                  ↓
        └──────────────┬──────────────────┘
                       ↓
            Validación Server (salas-validation) ← ✅ Mismas reglas
                       ↓
               Lógica de Negocio
```

---

## ✨ Conclusión

Sí, **`salas-validation` funciona para REST, gRPC y cualquier protocolo**. Las validaciones son independientes del transporte.

**Recomendación:**
- Si ya tienes Proto messages → Úsalos directamente o convierte DTOs a Proto
- Si necesitas DTOs REST específicos → Implementa conversiones `From<DTO> for Proto`
- Usa el extractor `ValidatedJson<T>` para automatizar en Axum

¡Todo centralizado, type-safe y con los mismos mensajes! 🎉
