# Sistema de Reservas de Salas

Implementación básica de un sistema de gestión de salas siguiendo principios de **Vertical Slice Architecture** y **Clean Architecture** en Rust.

## Arquitectura

El proyecto está organizado en capas siguiendo principios de arquitectura limpia:

```
crates/
├── app/                          # Aplicación principal (servidor HTTP)
└── features/
    └── salas/
        ├── domain/               # Lógica de negocio pura
        ├── application/          # Casos de uso y puertos
        ├── infrastructure/       # Implementaciones (repositorio in-memory)
        └── api/                  # DTOs y mappers
```

### Capas

- **Domain**: Entidades y lógica de negocio sin dependencias externas
- **Application**: Servicios de aplicación y definición de puertos (interfaces)
- **Infrastructure**: Implementaciones concretas (repositorio in-memory)
- **API**: DTOs para la capa HTTP y mappers

## Características

- ✅ CRUD básico de salas
- ✅ Validaciones en el dominio
- ✅ Manejo de errores tipado
- ✅ Repositorio in-memory
- ✅ API REST con Axum
- ✅ Tests unitarios

## Endpoints

### Crear sala
```bash
xh POST http://localhost:3000/salas
Content-Type: application/json

{
  "nombre": "Sala de Conferencias",
  "capacidad": 20
}
```

### Listar salas
```bash
xh GET http://localhost:3000/salas
```

### Obtener sala por ID
```bash
xh GET http://localhost:3000/salas/{id}
```

### Activar sala
```bash
xh PUT http://localhost:3000/salas/{id}/activar
```

### Desactivar sala
```bash
PUT http://localhost:3000/salas/{id}/desactivar
```

## Uso

### Compilar
```bash
cargo build
```

### Ejecutar tests
```bash
cargo test
```

### Ejecutar servidor
```bash
cargo run
```

El servidor se iniciará en `http://127.0.0.1:3000`

## Reglas de negocio

- El nombre de una sala no puede estar vacío
- El nombre no puede exceder 100 caracteres
- La capacidad debe estar entre 1 y 1000
- Las salas se crean activas por defecto
- Las salas pueden activarse/desactivarse

## Tecnologías

- **Rust** - Lenguaje de programación
- **Axum** - Framework web
- **Tokio** - Runtime asíncrono
- **Serde** - Serialización/deserialización
- **UUID** - Generación de identificadores
