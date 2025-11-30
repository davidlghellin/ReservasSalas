# Arquitectura del Sistema de Validación

## 🏗️ Visión General

```
┌─────────────────────────────────────────────────────────────────┐
│                    CAPA DE PRESENTACIÓN                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │   Iced   │  │  Dioxus  │  │  Slint   │  │   CLI    │       │
│  │  (gRPC)  │  │  (REST)  │  │  (REST)  │  │  (Args)  │       │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘       │
│       │             │              │              │             │
│       │ Proto       │ DTO→Proto    │ DTO→Proto    │ Directa     │
│       ▼             ▼              ▼              ▼             │
│  ┌────────────────────────────────────────────────────────┐    │
│  │         CrearSalaRequest (Proto Message)               │    │
│  └────────────────────────────────────────────────────────┘    │
│                           │                                     │
└───────────────────────────┼─────────────────────────────────────┘
                            │
                            │ .validar()
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│              SALAS-VALIDATION (Capa Compartida)                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌────────────────────────────────────────────────────┐        │
│  │  ValidarSala trait                                  │        │
│  │  • validar() -> Result<(), Error>                  │        │
│  └────────────────────────────────────────────────────┘        │
│                           │                                     │
│  ┌────────────────────────┼───────────────────────────┐        │
│  │                        │                            │        │
│  ▼                        ▼                            ▼        │
│  validar_nombre()    validar_capacidad()      validar_id()     │
│  • 3-100 chars       • 1-500 personas         • UUID válido    │
│  • Alfanuméricos     • Mayor que 0                             │
│  • No vacío                                                     │
│                                                                  │
│  ┌────────────────────────────────────────────────────┐        │
│  │  SalaValidationError (enum)                        │        │
│  │  • NombreVacio                                      │        │
│  │  • NombreLongitudInvalida { min, max, actual }    │        │
│  │  • CapacidadCero                                   │        │
│  │  • mensaje_usuario() → String amigable             │        │
│  └────────────────────────────────────────────────────┘        │
│                                                                  │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            │ Usado por backend
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                      CAPA DE BACKEND                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────┐              ┌──────────────────┐        │
│  │  Servidor gRPC   │              │   API REST       │        │
│  │  (Tonic)         │              │   (Axum)         │        │
│  └────────┬─────────┘              └────────┬─────────┘        │
│           │                                  │                  │
│           │ request.validar()?               │ dto→proto        │
│           │                                  │ request.validar()│
│           ▼                                  ▼                  │
│  ┌────────────────────────────────────────────────────┐        │
│  │         Lógica de Negocio (Domain Layer)           │        │
│  └────────────────────────────────────────────────────┘        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## 🔄 Flujo de Validación

### 1. Validación Client-Side (UX)

```
Usuario escribe "AB" en input de nombre
          ↓
Frontend: request.validar()
          ↓
Error: NombreLongitudInvalida { min: 3, max: 100, actual: 2 }
          ↓
UI muestra: "El nombre debe tener entre 3 y 100 caracteres. Actualmente tiene 2"
          ↓
❌ Request NO se envía al backend
```

### 2. Validación Server-Side (Seguridad)

```
Cliente malicioso envía request directo con grpcurl
          ↓
Backend recibe CrearSalaRequest { nombre: "AB", capacidad: 50 }
          ↓
Servidor: request.validar()
          ↓
Error: NombreLongitudInvalida { ... }
          ↓
Response: Status::InvalidArgument("El nombre debe tener...")
          ↓
❌ Request rechazado ANTES de llegar a la lógica de negocio
```

## 📦 Estructura del Crate

```
salas-validation/
│
├── Cargo.toml                  # Dependencias (salas-grpc, thiserror, uuid)
│
├── src/
│   ├── lib.rs                  # Exports públicos
│   │   └── pub use {
│   │       ValidarSala,
│   │       SalaValidationError,
│   │       validar_nombre,
│   │       validar_capacidad,
│   │       validar_id,
│   │       NOMBRE_MIN_LENGTH,
│   │       ...
│   │   }
│   │
│   ├── error.rs                # Definición de errores
│   │   └── SalaValidationError enum
│   │       ├── NombreVacio
│   │       ├── NombreLongitudInvalida { min, max, actual }
│   │       ├── CapacidadCero
│   │       └── mensaje_usuario() → String
│   │
│   └── sala.rs                 # Trait + implementaciones
│       ├── ValidarSala trait
│       │   └── fn validar(&self) -> Result<()>
│       │
│       ├── impl ValidarSala for CrearSalaRequest
│       ├── impl ValidarSala for ActivarSalaRequest
│       ├── impl ValidarSala for DesactivarSalaRequest
│       │
│       ├── pub fn validar_nombre(nombre: &str) -> Result<()>
│       ├── pub fn validar_capacidad(capacidad: u32) -> Result<()>
│       └── pub fn validar_id(id: &str) -> Result<()>
│
├── README.md                   # Documentación general
├── EJEMPLOS.md                 # Ejemplos de uso
├── BACKEND_INTEGRATION.md      # Integración en backend
├── REST_INTEGRATION.md         # Integración con REST
└── ARQUITECTURA.md            # Este documento
```

## 🎯 Decisiones de Diseño

### ¿Por qué Protocol Buffers como base?

```
✅ VENTAJAS:
• Proto messages ya existen (definidos en .proto)
• Type-safe: tipos generados automáticamente
• Serializables: funcionan con gRPC y JSON (REST)
• Versionables: compatible con evolución del schema

❌ ALTERNATIVAS DESCARTADAS:
• Structs Rust custom → Duplicaría definiciones
• Validar DTOs REST directamente → No reutilizable con gRPC
• Validator crate solo → No compartible entre proto y DTOs
```

### ¿Por qué trait `ValidarSala` en lugar de métodos en Proto?

```
✅ VENTAJAS:
• Proto messages son generados (no modificables)
• Trait es extensible a cualquier tipo (DTOs REST, etc.)
• Separación de concerns: Proto = datos, Validation = reglas

❌ ALTERNATIVAS:
• Modificar proto generated code → Se pierde en regeneración
• Macros en proto → Complica build.rs y mantenimiento
```

### ¿Por qué funciones públicas `validar_*()` además del trait?

```
✅ VENTAJAS:
• Reutilizables en DTOs REST sin conversión
• Validación granular campo por campo
• Útiles en CLI y testing unitario
• Hints de UI (mostrar reglas antes de validar)

📝 EJEMPLO:
let nombre = get_user_input();
if let Err(e) = validar_nombre(&nombre) {
    // Mostrar error inmediatamente sin crear struct completo
}
```

## 🔌 Integraciones

### Frontend con gRPC (Iced, CLI)

```rust
// Directo - Proto messages ya existen
request.validar()?
```

### Frontend con REST (Dioxus, Web)

```rust
// Opción 1: Validar campos individuales
validar_nombre(&dto.nombre)?;
validar_capacidad(dto.capacidad)?;

// Opción 2: Convertir DTO → Proto
let request: CrearSalaRequest = dto.into();
request.validar()?;
```

### Backend gRPC

```rust
// Interceptor o handler directo
req.validar().map_err(|e| Status::invalid_argument(e.to_string()))?;
```

### Backend REST

```rust
// Convertir DTO → Proto
let request: CrearSalaRequest = dto.into();
request.validar().map_err(|e| (StatusCode::BAD_REQUEST, e.mensaje_usuario()))?;
```

## 📊 Comparación con Alternativas

| Enfoque | DRY | Type-safe | Proto+REST | Mensajes custom |
|---------|-----|-----------|------------|-----------------|
| **salas-validation** (✅) | ✅ | ✅ | ✅ | ✅ |
| Validator crate | ⚠️ | ✅ | ❌ | ⚠️ |
| protoc-gen-validate | ⚠️ | ✅ | ✅ | ❌ |
| Validación manual | ❌ | ⚠️ | ⚠️ | ✅ |

**Legend:**
- ✅ Excelente
- ⚠️ Parcial/Requiere trabajo extra
- ❌ No soportado

## 🧪 Testing Strategy

```
Unit Tests (14 tests en sala.rs)
    ↓
Testing reglas individuales:
• validar_nombre()
• validar_capacidad()
• validar_id()
    ↓
Testing implementaciones de trait:
• CrearSalaRequest.validar()
• ActivarSalaRequest.validar()
    ↓
Integration Tests (en frontends/backends)
• Test que frontend rechaza requests inválidos
• Test que backend rechaza requests inválidos
• Test consistencia de mensajes de error
```

## 🚀 Evolución Futura

### Fase 1: ✅ Completado
- [x] Trait `ValidarSala`
- [x] Implementaciones para Sala requests
- [x] Funciones públicas reutilizables
- [x] Tests unitarios
- [x] Documentación

### Fase 2: Futuro
- [ ] Validaciones para Reservas
- [ ] Validaciones para Usuarios
- [ ] Validaciones asíncronas (DB checks)
- [ ] Validaciones contextuales (ej: verificar disponibilidad)

### Fase 3: Avanzado
- [ ] Macro `#[derive(ValidarSala)]` para generación automática
- [ ] Integration con OpenTelemetry para métricas de validación
- [ ] Validaciones condicionales (reglas diferentes por rol)

## 💡 Principios de Diseño

1. **DRY (Don't Repeat Yourself)**
   - Una sola implementación para todas las capas

2. **Type Safety**
   - Errores en compile-time, no runtime

3. **Separation of Concerns**
   - Proto = estructura de datos
   - Validation = reglas de negocio

4. **User Experience**
   - Mensajes claros y accionables
   - Validación client-side para feedback inmediato

5. **Security**
   - Validación server-side siempre
   - No confiar en el cliente

6. **Maintainability**
   - Cambios en un solo lugar
   - Documentación y ejemplos completos

---

**Resultado:** Sistema robusto, type-safe y mantenible ✅
