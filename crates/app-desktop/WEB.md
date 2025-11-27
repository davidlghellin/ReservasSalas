# Versión Web - App Desktop

Tu aplicación tiene **dos opciones** para ejecutarse en el navegador:

## 🌐 Opción 1: Usar `app-web` (Recomendado)

Ya tienes una versión web completamente funcional en `crates/app-web/` que usa:
- **Axum** como servidor
- **Askama** para templates HTML
- **SSR** (Server-Side Rendering)

### Ejecutar:
```bash
cd /Users/davidlopez/Proyectos/ReservasSalas
cargo run --bin server

# Abrir en navegador:
# http://localhost:3000
```

Esta versión es la **más robusta** porque:
- ✅ Comparte el mismo código de negocio
- ✅ Renderizado del servidor (mejor SEO)
- ✅ Funciona sin JavaScript si es necesario
- ✅ Ya está completamente integrada

---

## 🎨 Opción 2: Convertir el frontend de Tauri en SPA

Puedes usar el mismo frontend HTML/CSS/JS de la app desktop y conectarlo directamente a la API REST.

### Estructura:

```
crates/app-desktop-web/
├── index.html    (copiar desde app-desktop/src/)
├── styles.css    (copiar desde app-desktop/src/)
└── main.js       (modificar para usar fetch en lugar de Tauri)
```

### Pasos:

#### 1. Crear directorio web

```bash
mkdir -p crates/app-desktop-web
```

#### 2. Copiar archivos del frontend

```bash
cp crates/app-desktop/src/index.html crates/app-desktop-web/
cp crates/app-desktop/src/styles.css crates/app-desktop-web/
cp crates/app-desktop/src/main.js crates/app-desktop-web/
```

#### 3. Modificar `main.js` para usar fetch

Reemplazar las llamadas a `invoke()` por `fetch()`:

**Antes (Tauri):**
```javascript
const salas = await invoke('listar_salas');
```

**Después (Web):**
```javascript
const response = await fetch('http://localhost:3000/api/salas');
const salas = await response.json();
```

#### 4. Servir con cualquier servidor HTTP

**Opción A: Python**
```bash
cd crates/app-desktop-web
python3 -m http.server 8080

# Abrir: http://localhost:8080
```

**Opción B: Node.js (http-server)**
```bash
npm install -g http-server
cd crates/app-desktop-web
http-server -p 8080

# Abrir: http://localhost:8080
```

**Opción C: Rust (simple-http-server)**
```bash
cargo install simple-http-server
cd crates/app-desktop-web
simple-http-server -p 8080

# Abrir: http://localhost:8080
```

---

## 🚀 Crear versión web optimizada

Si quieres una SPA moderna con build optimizado:

### Con Vite (Recomendado)

#### 1. Crear proyecto

```bash
cd crates
npm create vite@latest app-desktop-spa -- --template vanilla

cd app-desktop-spa
npm install
```

#### 2. Copiar assets

```bash
cp ../app-desktop/src/styles.css src/
```

#### 3. Crear `src/main.js`:

```javascript
import './styles.css'

// API base URL
const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:3000/api';

// Funciones de API
async function listarSalas() {
  const response = await fetch(`${API_BASE}/salas`);
  if (!response.ok) throw new Error('Error al listar salas');
  return response.json();
}

async function crearSala(nombre, capacidad) {
  const response = await fetch(`${API_BASE}/salas`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ nombre, capacidad })
  });
  if (!response.ok) throw new Error('Error al crear sala');
  return response.json();
}

async function activarSala(id) {
  const response = await fetch(`${API_BASE}/salas/${id}/activar`, {
    method: 'PUT'
  });
  if (!response.ok) throw new Error('Error al activar sala');
  return response.json();
}

async function desactivarSala(id) {
  const response = await fetch(`${API_BASE}/salas/${id}/desactivar`, {
    method: 'PUT'
  });
  if (!response.ok) throw new Error('Error al desactivar sala');
  return response.json();
}

// El resto del código...
// (copiar lógica de app-desktop/src/main.js)
```

#### 4. Compilar para producción

```bash
npm run build

# Los archivos optimizados estarán en dist/
```

#### 5. Servir la versión compilada

```bash
npm run preview
# o
cd dist && python3 -m http.server 8080
```

---

## 📦 Integrar en el servidor Axum

Puedes servir la SPA directamente desde tu servidor Rust:

### 1. Modificar `crates/app/src/main.rs`:

```rust
use tower_http::services::ServeDir;

// ...

let app = Router::new()
    .merge(web_router)
    .nest("/api", api_router)
    .nest_service("/desktop-spa", ServeDir::new("crates/app-desktop-spa/dist"))
    .layer(cors);
```

### 2. Acceder:

- **REST API**: http://localhost:3000/api/salas
- **Web SSR**: http://localhost:3000/salas
- **Desktop SPA**: http://localhost:3000/desktop-spa

---

## 🌍 Desplegar en producción

### Netlify / Vercel

```bash
# En el directorio de la SPA
npm run build

# Subir la carpeta dist/
```

**netlify.toml**:
```toml
[build]
  command = "npm run build"
  publish = "dist"

[[redirects]]
  from = "/api/*"
  to = "https://tu-backend.com/api/:splat"
  status = 200
```

### Docker (Frontend + Backend)

```dockerfile
# Dockerfile en la raíz del proyecto
FROM rust:latest as backend-builder

WORKDIR /app
COPY . .
RUN cargo build --release --bin server

FROM node:20 as frontend-builder

WORKDIR /app
COPY crates/app-desktop-spa/package*.json ./
RUN npm install
COPY crates/app-desktop-spa ./
RUN npm run build

FROM debian:bookworm-slim

COPY --from=backend-builder /app/target/release/server /usr/local/bin/
COPY --from=frontend-builder /app/dist /var/www/html

EXPOSE 3000

CMD ["server"]
```

---

## 🎯 Comparación de opciones

| Característica | app-web (SSR) | Desktop SPA | Tauri Desktop |
|---------------|---------------|-------------|---------------|
| Renderizado | Servidor | Cliente | Nativo |
| JavaScript | Opcional | Requerido | Requerido |
| SEO | Excelente | Limitado | N/A |
| Offline | No | Service Worker | Sí |
| Tamaño | ~1MB | ~500KB | ~10MB |
| Instalación | No | No | Sí |
| Notificaciones | Limitadas | Web Push | Nativas |

---

## 🔗 Recursos

- [Vite](https://vitejs.dev/)
- [SPA vs SSR](https://web.dev/rendering-on-the-web/)
- [Service Workers](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API)

---

## 💡 Recomendación

**Para tu caso de uso:**

1. **Producción web**: Usa `app-web` (SSR con Axum) ✅
2. **App móvil**: Compila APK con Tauri Android
3. **App escritorio**: Usa Tauri (macOS, Linux, Windows)
4. **Prototipo rápido**: Copia el frontend y usa Python http.server

Todas las opciones comparten el mismo backend REST API en `http://localhost:3000/api`.
