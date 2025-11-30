# 📡 Documentación de API

## Autenticación

Todas las APIs (REST y gRPC) requieren autenticación JWT excepto el endpoint de login.

**Formato del token:**
```
Authorization: Bearer <token>
```

Para gRPC, usar metadata header:
```
authorization: Bearer <token>
```

---

## 🔐 Autenticación (Usuarios)

### gRPC - Login
```protobuf
service UsuarioService {
  rpc Login(LoginRequest) returns (LoginResponse);
}

message LoginRequest {
  string email = 1;
  string password = 2;
}

message LoginResponse {
  string token = 1;
  UsuarioPublico usuario = 2;
}
```

**Ejemplo:**
```bash
grpcurl -plaintext -d '{
  "email": "admin@reservas.com",
  "password": "admin123"
}' localhost:50051 usuario.UsuarioService/Login
```

**Respuesta:**
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "usuario": {
    "id": "b9b6d22f-...",
    "nombre": "Administrador",
    "email": "admin@reservas.com",
    "rol": "Admin",
    "createdAt": "2025-11-30T10:49:05Z"
  }
}
```

### REST - Login
```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "admin@reservas.com",
  "password": "admin123"
}
```

**Respuesta:**
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "usuario": {
    "id": "b9b6d22f-...",
    "nombre": "Administrador",
    "email": "admin@reservas.com",
    "rol": "Admin"
  }
}
```

### gRPC - Registro
```protobuf
rpc Register(RegisterRequest) returns (RegisterResponse);

message RegisterRequest {
  string nombre = 1;
  string email = 2;
  string password = 3;
}
```

**Ejemplo:**
```bash
grpcurl -plaintext -d '{
  "nombre": "Juan Pérez",
  "email": "juan@example.com",
  "password": "password123"
}' localhost:50051 usuario.UsuarioService/Register
```

### REST - Registro
```http
POST /api/auth/register
Content-Type: application/json

{
  "nombre": "Juan Pérez",
  "email": "juan@example.com",
  "password": "password123"
}
```

---

## 🏢 Salas

### gRPC - Listar Salas
```protobuf
rpc ListarSalas(ListarSalasRequest) returns (ListarSalasResponse);

message ListarSalasRequest {}

message ListarSalasResponse {
  repeated SalaResponse salas = 1;
}

message SalaResponse {
  string id = 1;
  string nombre = 2;
  uint32 capacidad = 3;
  bool activa = 4;
}
```

**Ejemplo:**
```bash
grpcurl -plaintext \
  -H "authorization: Bearer TU_TOKEN" \
  -d '{}' \
  localhost:50051 sala.SalaService/ListarSalas
```

### REST - Listar Salas
```http
GET /api/salas
Authorization: Bearer <token>
```

**Respuesta:**
```json
[
  {
    "id": "a1b2c3d4-...",
    "nombre": "Sala de Conferencias",
    "capacidad": 20,
    "activa": true
  },
  {
    "id": "e5f6g7h8-...",
    "nombre": "Sala de Reuniones",
    "capacidad": 10,
    "activa": false
  }
]
```

### gRPC - Crear Sala
```protobuf
rpc CrearSala(CrearSalaRequest) returns (SalaResponse);

message CrearSalaRequest {
  string nombre = 1;
  uint32 capacidad = 2;
}
```

**Ejemplo:**
```bash
grpcurl -plaintext \
  -H "authorization: Bearer TU_TOKEN" \
  -d '{
    "nombre": "Sala de Conferencias",
    "capacidad": 20
  }' \
  localhost:50051 sala.SalaService/CrearSala
```

### REST - Crear Sala
```http
POST /api/salas
Authorization: Bearer <token>
Content-Type: application/json

{
  "nombre": "Sala de Conferencias",
  "capacidad": 20
}
```

### gRPC - Obtener Sala
```protobuf
rpc ObtenerSala(ObtenerSalaRequest) returns (SalaResponse);

message ObtenerSalaRequest {
  string id = 1;
}
```

**Ejemplo:**
```bash
grpcurl -plaintext \
  -H "authorization: Bearer TU_TOKEN" \
  -d '{"id": "SALA_ID"}' \
  localhost:50051 sala.SalaService/ObtenerSala
```

### REST - Obtener Sala
```http
GET /api/salas/{id}
Authorization: Bearer <token>
```

### gRPC - Activar Sala
```protobuf
rpc ActivarSala(ActivarSalaRequest) returns (SalaResponse);

message ActivarSalaRequest {
  string id = 1;
}
```

**Ejemplo:**
```bash
grpcurl -plaintext \
  -H "authorization: Bearer TU_TOKEN" \
  -d '{"id": "SALA_ID"}' \
  localhost:50051 sala.SalaService/ActivarSala
```

### REST - Activar Sala
```http
PUT /api/salas/{id}/activar
Authorization: Bearer <token>
```

### gRPC - Desactivar Sala
```protobuf
rpc DesactivarSala(DesactivarSalaRequest) returns (SalaResponse);

message DesactivarSalaRequest {
  string id = 1;
}
```

**Ejemplo:**
```bash
grpcurl -plaintext \
  -H "authorization: Bearer TU_TOKEN" \
  -d '{"id": "SALA_ID"}' \
  localhost:50051 sala.SalaService/DesactivarSala
```

### REST - Desactivar Sala
```http
PUT /api/salas/{id}/desactivar
Authorization: Bearer <token>
```

---

## 🔒 Permisos

| Operación | Admin | Usuario |
|-----------|-------|---------|
| Login/Registro | ✅ | ✅ |
| Listar salas | ✅ | ✅ |
| Crear sala | ✅ | ✅ |
| Obtener sala | ✅ | ✅ |
| Activar sala | ✅ | ✅ |
| Desactivar sala | ✅ | ✅ |
| Listar usuarios | ✅ | ❌ |

---

## ⚠️ Códigos de Error

### gRPC Status Codes
- `UNAUTHENTICATED` - Token inválido o ausente
- `PERMISSION_DENIED` - Usuario sin permisos
- `NOT_FOUND` - Recurso no encontrado
- `INVALID_ARGUMENT` - Datos de entrada inválidos
- `INTERNAL` - Error interno del servidor

### REST HTTP Status
- `200 OK` - Operación exitosa
- `201 Created` - Recurso creado
- `400 Bad Request` - Datos inválidos
- `401 Unauthorized` - No autenticado
- `403 Forbidden` - Sin permisos
- `404 Not Found` - Recurso no encontrado
- `500 Internal Server Error` - Error del servidor

---

## 📝 Validaciones

### Salas
- **Nombre**: No vacío, máximo 100 caracteres
- **Capacidad**: Entre 1 y 1000

### Usuarios
- **Email**: Formato válido, único en el sistema
- **Contraseña**: Mínimo 8 caracteres
- **Nombre**: No vacío

---

## 🔧 Herramientas Útiles

### grpcurl
Instalar:
```bash
brew install grpcurl  # macOS
```

Listar servicios:
```bash
grpcurl -plaintext localhost:50051 list
```

### curl
Probar REST API:
```bash
# Con Pretty Print
curl http://localhost:3000/api/salas \
  -H "Authorization: Bearer TOKEN" | jq
```

### httpie (xh)
Alternativa moderna a curl:
```bash
xh http://localhost:3000/api/salas Authorization:"Bearer TOKEN"
```
