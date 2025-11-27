# Sala gRPC Service

Servidor gRPC para la gestión de salas, implementado como un adaptador sobre el `SalaService` de la capa de aplicación.

## Arquitectura

Este crate actúa como un **puerto de entrada** en la arquitectura hexagonal:

```
┌─────────────────┐
│   gRPC Client   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  SalaGrpcServer │  ◄── Este crate (adaptador)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  SalaService    │  ◄── Application layer
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│     Domain      │
└─────────────────┘
```

## Definición del servicio

El archivo `proto/sala.proto` define el contrato gRPC:

- **CrearSala**: Crea una nueva sala
- **ObtenerSala**: Obtiene una sala por ID
- **ListarSalas**: Lista todas las salas
- **ActivarSala**: Activa una sala
- **DesactivarSala**: Desactiva una sala

## Uso con grpcurl

El servidor tiene habilitada la **reflexión de gRPC**, lo que permite usar `grpcurl` sin necesidad de los archivos `.proto`.

### Requisitos previos

Instalar grpcurl:
```bash
# macOS
brew install grpcurl

# Linux
go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest

# Más opciones: https://github.com/fullstorydev/grpcurl
```

### Comandos disponibles

#### 1. Listar servicios disponibles
```bash
grpcurl -plaintext localhost:50051 list
```

Salida:
```
grpc.reflection.v1.ServerReflection
sala.SalaService
```

#### 2. Listar métodos de SalaService
```bash
grpcurl -plaintext localhost:50051 list sala.SalaService
```

Salida:
```
sala.SalaService.ActivarSala
sala.SalaService.CrearSala
sala.SalaService.DesactivarSala
sala.SalaService.ListarSalas
sala.SalaService.ObtenerSala
```

#### 3. Crear una sala
```bash
grpcurl -plaintext -d '{
  "nombre": "Sala Conferencias",
  "capacidad": 50
}' localhost:50051 sala.SalaService/CrearSala
```

Respuesta:
```json
{
  "id": "442ffc97-d0f3-46d5-8292-ee0fd12a646f",
  "nombre": "Sala Conferencias",
  "capacidad": 50,
  "activa": true
}
```

#### 4. Listar todas las salas
```bash
grpcurl -plaintext -d '{}' localhost:50051 sala.SalaService/ListarSalas
```

Respuesta:
```json
{
  "salas": [
    {
      "id": "442ffc97-d0f3-46d5-8292-ee0fd12a646f",
      "nombre": "Sala Conferencias",
      "capacidad": 50,
      "activa": true
    },
    {
      "id": "8f500cdb-4a63-4a5f-80b5-6c888d33a0c0",
      "nombre": "Sala Reuniones",
      "capacidad": 20,
      "activa": true
    }
  ]
}
```

#### 5. Obtener una sala por ID
```bash
grpcurl -plaintext -d '{
  "id": "442ffc97-d0f3-46d5-8292-ee0fd12a646f"
}' localhost:50051 sala.SalaService/ObtenerSala
```

Respuesta:
```json
{
  "id": "442ffc97-d0f3-46d5-8292-ee0fd12a646f",
  "nombre": "Sala Conferencias",
  "capacidad": 50,
  "activa": true
}
```

#### 6. Activar una sala
```bash
grpcurl -plaintext -d '{
  "id": "442ffc97-d0f3-46d5-8292-ee0fd12a646f"
}' localhost:50051 sala.SalaService/ActivarSala
```

Respuesta:
```json
{
  "id": "442ffc97-d0f3-46d5-8292-ee0fd12a646f",
  "nombre": "Sala Conferencias",
  "capacidad": 50,
  "activa": true
}
```

#### 7. Desactivar una sala
```bash
grpcurl -plaintext -d '{
  "id": "442ffc97-d0f3-46d5-8292-ee0fd12a646f"
}' localhost:50051 sala.SalaService/DesactivarSala
```

Respuesta:
```json
{
  "id": "442ffc97-d0f3-46d5-8292-ee0fd12a646f",
  "nombre": "Sala Conferencias",
  "capacidad": 50,
  "activa": false
}
```

## Manejo de errores

El servidor convierte los errores del dominio (`SalaError`) a códigos de estado gRPC:

| Error del dominio | Código gRPC |
|-------------------|-------------|
| `NombreVacio`, `NombreDemasiadoLargo`, `CapacidadInvalida` | `INVALID_ARGUMENT` |
| `NoEncontrada` | `NOT_FOUND` |
| `ErrorRepositorio` | `INTERNAL` |
| `Validacion` | `INVALID_ARGUMENT` |

Ejemplo de error:
```bash
grpcurl -plaintext -d '{
  "nombre": "",
  "capacidad": 50
}' localhost:50051 sala.SalaService/CrearSala
```

Respuesta:
```
ERROR:
  Code: InvalidArgument
  Message: El nombre no puede estar vacío
```

## Desarrollo

### Compilar el proyecto
```bash
cargo build
```

El archivo `build.rs` compila automáticamente el `.proto` y genera:
- Código Rust para los mensajes y servicios
- Descriptor set para reflexión gRPC

### Ejecutar el servidor
```bash
cargo run --bin server
```

El servidor gRPC escucha en `localhost:50051` junto con:
- HTTP/REST API en `localhost:3000/api`
- Web UI en `localhost:3000`
- Swagger UI en `localhost:3000/api/swagger-ui`

### Modificar el servicio gRPC

#### 1. Editar el archivo `.proto`

Para añadir campos o funcionalidad, edita `proto/sala.proto`. Por ejemplo, para añadir un campo `descripcion`:

```protobuf
message SalaResponse {
  string id = 1;
  string nombre = 2;
  uint32 capacidad = 3;
  bool activa = 4;
  string descripcion = 5;  // ← Nuevo campo
}
```

#### 2. Compilación automática

**No necesitas ejecutar nada manualmente**. El archivo `build.rs` se ejecuta automáticamente cuando haces:

```bash
cargo build
```

Esto regenera el código Rust a partir del `.proto` actualizado.

#### 3. Actualizar el código Rust

Después de compilar, actualiza `src/server.rs` para usar los nuevos campos:

```rust
Ok(Response::new(SalaResponse {
    id: sala.id().to_string(),
    nombre: sala.nombre().to_string(),
    capacidad: sala.capacidad(),
    activa: sala.activa,
    descripcion: sala.descripcion().to_string(), // ← Nuevo campo
}))
```

#### 4. Añadir nuevos métodos RPC

Si añades un nuevo RPC en `proto/sala.proto`:

```protobuf
service SalaService {
  // ... métodos existentes
  rpc EliminarSala(EliminarSalaRequest) returns (SalaResponse);
}

message EliminarSalaRequest {
  string id = 1;
}
```

Implementa el método en `src/server.rs`:

```rust
#[tonic::async_trait]
impl SalaServiceTrait for SalaGrpcServer {
    // ... métodos existentes

    async fn eliminar_sala(
        &self,
        request: Request<EliminarSalaRequest>,
    ) -> Result<Response<SalaResponse>, Status> {
        let req = request.into_inner();

        // Tu implementación aquí
        let sala = self.service
            .eliminar_sala(&req.id)
            .await
            .map_err(sala_error_to_status)?;

        Ok(Response::new(SalaResponse {
            id: sala.id().to_string(),
            nombre: sala.nombre().to_string(),
            capacidad: sala.capacidad(),
            activa: sala.activa,
        }))
    }
}
```

#### Flujo completo

```
1. Editar proto/sala.proto
         ↓
2. cargo build  (compila automáticamente el .proto)
         ↓
3. Actualizar src/server.rs con los nuevos tipos/métodos
         ↓
4. cargo build  (compila tu código Rust)
         ↓
5. cargo run --bin server  (listo!)
```

**Importante**: No necesitas invocar `protoc` manualmente. `tonic-build` se encarga de todo automáticamente.

## Dependencias

- `tonic 0.12` - Framework gRPC
- `tonic-build 0.12` - Compilación de .proto
- `tonic-reflection 0.12` - Reflexión para grpcurl
- `prost 0.13` - Serialización Protocol Buffers

## Referencias

- [Tonic Documentation](https://docs.rs/tonic/)
- [Protocol Buffers](https://protobuf.dev/)
- [grpcurl GitHub](https://github.com/fullstorydev/grpcurl)
