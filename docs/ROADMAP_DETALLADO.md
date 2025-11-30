# 🗺️ Roadmap - Sistema de Reservas de Salas

Hoja de ruta incremental para añadir features al proyecto.

---

## ✅ Fundamentos completados

- [x] Arquitectura hexagonal (Domain, Application, Infrastructure)
- [x] gRPC con Tonic + Protocol Buffers
- [x] REST API con Axum
- [x] Validaciones compartidas (salas-validation)
- [x] Persistencia JSON (FileSalaRepository)
- [x] Persistencia en memoria (InMemorySalaRepository)
- [x] Frontend Iced con gRPC
- [x] Notificaciones del sistema (notify-rust)
- [x] CRUD completo de Salas

---

## 🎯 Fase 1: Dominio Core - Reservas (1-2 semanas)

### 1.1 Entidad Reserva ⭐ PRIORIDAD ALTA

**¿Qué es?** El corazón del sistema. Una reserva conecta una Sala con un Usuario en un periodo de tiempo.

**Estructura:**
```rust
// crates/features/reservas/domain/src/reserva.rs
pub struct Reserva {
    pub id: String,
    pub sala_id: String,          // FK a Sala
    pub usuario: String,           // Nombre o email del usuario
    pub fecha_inicio: DateTime<Utc>,
    pub fecha_fin: DateTime<Utc>,
    pub estado: EstadoReserva,     // Confirmada, Pendiente, Cancelada
    pub descripcion: Option<String>,
}

pub enum EstadoReserva {
    Pendiente,
    Confirmada,
    Cancelada,
}
```

**Validaciones:**
- Fecha fin > Fecha inicio
- No solapar con otras reservas de la misma sala
- Sala debe existir y estar activa
- Máximo X horas de reserva

**Tareas:**
- [ ] Crear `crates/features/reservas/domain`
- [ ] Definir entidad Reserva con validaciones
- [ ] Crear ReservaRepository trait
- [ ] Tests unitarios del dominio

**Tiempo estimado:** 2-3 días

---

### 1.2 Application Layer - Casos de uso

**Casos de uso principales:**
```rust
// crates/features/reservas/application/src/service.rs
pub trait ReservaService {
    async fn crear_reserva(&self, ...) -> Result<Reserva, ReservaError>;
    async fn listar_reservas_sala(&self, sala_id: &str) -> Result<Vec<Reserva>>;
    async fn listar_reservas_usuario(&self, usuario: &str) -> Result<Vec<Reserva>>;
    async fn cancelar_reserva(&self, id: &str) -> Result<Reserva>;
    async fn verificar_disponibilidad(&self, sala_id: &str, inicio: DateTime, fin: DateTime) -> Result<bool>;
}
```

**Lógica de negocio:**
- Verificar que la sala existe y está activa
- Detectar solapamientos (reservas conflictivas)
- Validar horarios permitidos (ej: 8am-10pm)
- Enviar notificaciones cuando se crea/cancela reserva

**Tareas:**
- [ ] Crear ReservaService trait
- [ ] Implementar ReservaServiceImpl
- [ ] Lógica de detección de conflictos
- [ ] Tests de casos de uso

**Tiempo estimado:** 3-4 días

---

### 1.3 Infrastructure - Persistencia

**Adaptadores:**
```rust
// FileReservaRepository (JSON)
// Estructura del JSON:
{
  "reservas": {
    "uuid-123": {
      "id": "uuid-123",
      "sala_id": "sala-uuid",
      "usuario": "david@example.com",
      "fecha_inicio": "2024-01-15T10:00:00Z",
      "fecha_fin": "2024-01-15T12:00:00Z",
      "estado": "Confirmada"
    }
  }
}
```

**Tareas:**
- [ ] Crear FileReservaRepository
- [ ] Implementar queries eficientes (buscar por sala, por usuario, por fecha)
- [ ] Tests de persistencia

**Tiempo estimado:** 2 días

---

### 1.4 gRPC API

**Protocol Buffer:**
```protobuf
// proto/reserva.proto
service ReservaService {
  rpc CrearReserva(CrearReservaRequest) returns (ReservaResponse);
  rpc ListarReservasSala(ListarReservasSalaRequest) returns (ListarReservasResponse);
  rpc CancelarReserva(CancelarReservaRequest) returns (ReservaResponse);
  rpc VerificarDisponibilidad(DisponibilidadRequest) returns (DisponibilidadResponse);
}

message CrearReservaRequest {
  string sala_id = 1;
  string usuario = 2;
  string fecha_inicio = 3; // ISO 8601
  string fecha_fin = 4;
  string descripcion = 5;
}
```

**Tareas:**
- [ ] Definir .proto para reservas
- [ ] Implementar servidor gRPC
- [ ] Integrar con ReservaService

**Tiempo estimado:** 1-2 días

---

### 1.5 Frontend Iced - Vista de Reservas

**Pantallas:**
1. **Lista de salas con disponibilidad**
   ```
   ┌─────────────────────────────────────┐
   │ Salas Disponibles                   │
   ├─────────────────────────────────────┤
   │ □ Sala 101 (Cap: 50)               │
   │   ✓ Disponible hoy 10:00-12:00     │
   │   [Ver Calendario]                  │
   │                                     │
   │ □ Sala 202 (Cap: 20)               │
   │   ✗ Ocupada hoy 10:00-14:00        │
   │   [Ver Calendario]                  │
   └─────────────────────────────────────┘
   ```

2. **Formulario de reserva**
   ```
   ┌─────────────────────────────────────┐
   │ Nueva Reserva - Sala 101            │
   ├─────────────────────────────────────┤
   │ Usuario: [________________]         │
   │ Fecha:   [15/01/2024 ▼]            │
   │ Hora inicio: [10:00 ▼]             │
   │ Hora fin:    [12:00 ▼]             │
   │ Descripción: [______________]       │
   │                                     │
   │ [Verificar disponibilidad]          │
   │ ✓ Horario disponible                │
   │                                     │
   │ [Cancelar] [Crear Reserva]          │
   └─────────────────────────────────────┘
   ```

3. **Calendario de reservas** (opcional avanzado)

**Tareas:**
- [ ] Vista lista de salas con disponibilidad
- [ ] Formulario crear reserva
- [ ] Vista mis reservas
- [ ] Cancelar reserva
- [ ] Widget date picker (o usar iced_aw)

**Tiempo estimado:** 4-5 días

**Dependencia adicional:**
```toml
[dependencies]
chrono = "0.4"  # Para manejo de fechas
iced_aw = "0.9" # Widgets adicionales (date picker, time picker)
```

---

## 🎯 Fase 2: Mejoras de UX/Productividad (1 semana)

### 2.1 Autenticación básica

**Objetivo:** Identificar quién hace cada reserva

**Implementación simple:**
```rust
// Sin base de datos de usuarios aún
// Solo pedir nombre/email al iniciar la app

struct App {
    usuario_actual: Option<String>,
    // ...
}

// Al crear reserva, usar usuario_actual
```

**Tareas:**
- [ ] Pantalla login/identificación
- [ ] Guardar usuario en estado de Iced
- [ ] Filtrar "Mis reservas" por usuario actual

**Tiempo estimado:** 1-2 días

---

### 2.2 Notificaciones de reserva

**Mejoras:**
- ✅ Notificación al crear reserva
- ✅ Notificación 15 min antes de una reserva (background task)
- ✅ Notificación al cancelar reserva

**Implementación:**
```rust
// Background task en Iced
Task::perform(
    verificar_proximas_reservas(),
    Message::ProximasReservas
)
```

**Tareas:**
- [ ] Polling de próximas reservas cada 5 minutos
- [ ] Mostrar notificación sistema cuando falta poco
- [ ] Guardar "ya notificado" para no repetir

**Tiempo estimado:** 1 día

---

### 2.3 Calendario visual (Widget)

**Objetivo:** Vista de calendario mensual con reservas

**Opciones:**
1. Usar `iced_aw::Calendar` (más fácil)
2. Crear widget custom (más control)

**Vista:**
```
┌────────────────────────────────────────┐
│  Enero 2024                            │
├────┬────┬────┬────┬────┬────┬────────┤
│ Lu │ Ma │ Mi │ Ju │ Vi │ Sa │ Do     │
├────┼────┼────┼────┼────┼────┼────────┤
│  1 │  2 │  3 │  4 │  5 │  6 │  7     │
│    │ 🟢 │ 🔴 │    │ 🟢 │    │        │
├────┼────┼────┼────┼────┼────┼────────┤
│  8 │  9 │ 10 │ 11 │ 12 │ 13 │ 14     │
│ 🔴 │    │ 🟢 │    │    │    │        │
└────┴────┴────┴────┴────┴────┴────────┘

🟢 Disponible  🔴 Reservado
```

**Tareas:**
- [ ] Integrar iced_aw::Calendar
- [ ] Colorear días según disponibilidad
- [ ] Click en día → crear reserva

**Tiempo estimado:** 2-3 días

---

## 🎯 Fase 3: Profesionalización (2 semanas)

### 3.1 Base de datos real (PostgreSQL)

**Objetivo:** Reemplazar JSON por PostgreSQL

**Implementación:**
```rust
// crates/features/salas/infrastructure/src/postgres_repository.rs
pub struct PostgresSalaRepository {
    pool: PgPool,
}

#[async_trait]
impl SalaRepository for PostgresSalaRepository {
    async fn guardar(&self, sala: &Sala) -> Result<(), SalaError> {
        sqlx::query!(
            "INSERT INTO salas (id, nombre, capacidad, activa) VALUES ($1, $2, $3, $4)",
            sala.id(),
            sala.nombre(),
            sala.capacidad() as i32,
            sala.esta_activa()
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
```

**Tareas:**
- [ ] Añadir sqlx + PostgreSQL
- [ ] Crear migrations (tablas salas, reservas)
- [ ] Implementar PostgresSalaRepository
- [ ] Implementar PostgresReservaRepository
- [ ] Tests de integración con Docker

**Dependencias:**
```toml
[dependencies]
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio", "uuid", "chrono"] }
```

**Tiempo estimado:** 3-4 días

---

### 3.2 Configuración por entorno

**Objetivo:** Diferentes configs para dev/prod

**Implementación:**
```toml
# config/dev.toml
[database]
url = "postgres://localhost/reservas_dev"

[server]
grpc_port = 50051
http_port = 3000

[files]
data_dir = "./data"
```

**Usar:**
```toml
[dependencies]
config = "0.14"
serde = { workspace = true, features = ["derive"] }
```

**Tareas:**
- [ ] Crear configs dev/prod
- [ ] Cargar config al inicio
- [ ] Variables de entorno override

**Tiempo estimado:** 1 día

---

### 3.3 Logs estructurados y métricas

**Objetivo:** Mejor observabilidad

**Implementación:**
```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(self))]
async fn crear_sala(&self, nombre: String, capacidad: u32) -> Result<Sala> {
    info!("Creando sala: {} con capacidad {}", nombre, capacidad);

    // ...

    info!(sala_id = %sala.id(), "Sala creada exitosamente");
    Ok(sala)
}
```

**Logs estructurados:**
```json
{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"Sala creada","sala_id":"abc-123"}
```

**Tareas:**
- [ ] Configurar tracing con JSON format
- [ ] Instrumentar funciones importantes
- [ ] Añadir métricas (contador de reservas, latencias)
- [ ] Opcional: integrar con Prometheus

**Tiempo estimado:** 1-2 días

---

### 3.4 Tests de integración

**Objetivo:** Testear el stack completo

**Ejemplo:**
```rust
#[tokio::test]
async fn test_crear_y_listar_reservas() {
    // 1. Setup: levantar servidor gRPC en puerto random
    let addr = start_test_server().await;

    // 2. Conectar cliente
    let mut client = ReservaServiceClient::connect(addr).await.unwrap();

    // 3. Crear sala
    let sala = client.crear_sala(...).await.unwrap();

    // 4. Crear reserva
    let reserva = client.crear_reserva(...).await.unwrap();

    // 5. Verificar que aparece en la lista
    let reservas = client.listar_reservas_sala(...).await.unwrap();
    assert_eq!(reservas.len(), 1);
}
```

**Tareas:**
- [ ] Tests end-to-end del flujo completo
- [ ] Mock del repositorio para tests rápidos
- [ ] CI/CD con GitHub Actions

**Tiempo estimado:** 2-3 días

---

## 🎯 Fase 4: Features Avanzadas (opcional)

### 4.1 Exportar reportes

- PDF de reservas del mes
- CSV para análisis
- Estadísticas de uso de salas

### 4.2 Recordatorios por email

- Integrar con SMTP
- Enviar email 24h antes de reserva

### 4.3 Gestión de usuarios completa

- Crear entidad Usuario
- Roles (admin, usuario normal)
- Límites de reserva por usuario

### 4.4 App móvil (Tauri Mobile)

- Reutilizar backend gRPC
- UI móvil con Tauri

### 4.5 Dashboard web (analytics)

- Gráficos de ocupación
- Salas más usadas
- Horas pico

---

## 📊 Resumen de tiempo estimado

| Fase | Descripción | Tiempo |
|------|-------------|--------|
| **Fase 1** | Dominio Reservas + gRPC + Iced | 1-2 semanas |
| **Fase 2** | UX (auth, notif, calendario) | 1 semana |
| **Fase 3** | PostgreSQL + Config + Tests | 2 semanas |
| **Fase 4** | Features avanzadas | Variable |

**Total mínimo viable (Fases 1-2):** ~3 semanas
**Producto robusto (Fases 1-3):** ~5 semanas

---

## 🎯 Próximo paso inmediato

**Recomendación:** Empezar por **Fase 1.1 - Entidad Reserva**

¿Quieres que te ayude a crear la entidad Reserva con sus validaciones?
