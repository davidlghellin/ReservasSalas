# Integración en Backend gRPC

Ejemplo de cómo integrar las validaciones en el servidor gRPC.

## 📦 Añadir dependencia

```toml
# crates/app/Cargo.toml (o donde esté tu servidor gRPC)
[dependencies]
salas-validation = { path = "crates/features/salas/validation" }
```

## 🔧 Implementación en el servidor

### Opción 1: Validar en cada handler

```rust
use salas_validation::ValidarSala;
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl SalaService for SalaServiceImpl {
    async fn crear_sala(
        &self,
        request: Request<CrearSalaRequest>,
    ) -> Result<Response<SalaResponse>, Status> {
        let req = request.into_inner();

        // ✅ Validar request
        req.validar()
            .map_err(|e| {
                // Convertir error de validación a Status gRPC
                Status::invalid_argument(e.to_string())
            })?;

        // Si pasa validación, continuar con la lógica de negocio
        let sala = self.repository.crear_sala(&req.nombre, req.capacidad).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(sala.into()))
    }

    async fn activar_sala(
        &self,
        request: Request<ActivarSalaRequest>,
    ) -> Result<Response<SalaResponse>, Status> {
        let req = request.into_inner();

        // ✅ Validar ID
        req.validar()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let sala = self.repository.activar_sala(&req.id).await
            .map_err(|e| Status::not_found(e.to_string()))?;

        Ok(Response::new(sala.into()))
    }
}
```

### Opción 2: Middleware de validación (más avanzado)

Crear un interceptor que valide automáticamente todos los requests:

```rust
use tonic::service::Interceptor;
use tonic::{Request, Status};
use salas_validation::ValidarSala;

pub struct ValidationInterceptor;

impl Interceptor for ValidationInterceptor {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        // Aquí podrías validar headers, autenticación, etc.
        Ok(request)
    }
}

// En el servidor
let svc = SalaServiceServer::with_interceptor(service, ValidationInterceptor);
```

### Opción 3: Función helper reutilizable

```rust
use salas_validation::{ValidarSala, SalaValidationError};
use tonic::Status;

/// Helper para convertir errores de validación a Status gRPC
fn validar_request<T: ValidarSala>(req: &T) -> Result<(), Status> {
    req.validar()
        .map_err(|e| Status::invalid_argument(e.to_string()))
}

// Uso en handlers
async fn crear_sala(
    &self,
    request: Request<CrearSalaRequest>,
) -> Result<Response<SalaResponse>, Status> {
    let req = request.into_inner();

    validar_request(&req)?;  // ✅ Una línea

    // Continuar con lógica...
}
```

## 🎯 Mensajes de error consistentes

Con esta integración, los errores de validación son consistentes en frontend y backend:

### Frontend (Iced)
```
❌ El nombre debe tener entre 3 y 100 caracteres. Actualmente tiene 2
```

### Backend (gRPC)
```
Status::invalid_argument("El nombre debe tener entre 3 y 100 caracteres (actual: 2)")
```

## 🔄 Flujo completo

```
Usuario en Iced Frontend
        ↓
Validación client-side (salas-validation)
        ↓ (si pasa)
Request gRPC → Backend
        ↓
Validación server-side (salas-validation) ← ✅ Mismas reglas
        ↓ (si pasa)
Lógica de negocio
        ↓
Response → Frontend
```

## 🧪 Testing en backend

Puedes testear que el servidor rechaza requests inválidos:

```rust
#[tokio::test]
async fn test_crear_sala_nombre_invalido() {
    let mut client = SalaServiceClient::connect("http://localhost:50051")
        .await
        .unwrap();

    let request = CrearSalaRequest {
        nombre: "AB".to_string(), // Muy corto
        capacidad: 50,
    };

    let response = client.crear_sala(request).await;

    // Debe fallar con INVALID_ARGUMENT
    assert!(response.is_err());
    assert_eq!(
        response.unwrap_err().code(),
        tonic::Code::InvalidArgument
    );
}
```

## 📊 Ventajas

| Ventaja | Descripción |
|---------|-------------|
| ✅ **Doble validación** | Client-side (UX) + server-side (seguridad) |
| ✅ **Mismo código** | No duplicar reglas entre frontend/backend |
| ✅ **Type-safe** | Errores en tiempo de compilación |
| ✅ **Mensajes consistentes** | Mismos errores en toda la app |
| ✅ **Fácil mantenimiento** | Cambiar regla en un solo lugar |

## 🚨 Importante

Aunque valides en el frontend, **SIEMPRE debes validar en el backend** por seguridad:
- El cliente podría estar manipulado
- Alguien podría hacer requests directos con grpcurl
- Validación server-side es la última línea de defensa

Con `salas-validation`, tienes ambas validaciones con el mismo código ✅
