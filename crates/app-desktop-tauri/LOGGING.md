# Sistema de Logging - App Desktop

El sistema de logging de la aplicación de escritorio guarda automáticamente todos los logs tanto en consola como en un archivo.

## 📍 Ubicación de los logs

### Por defecto
```
/tmp/tauri-app.log
```

### Personalizar la ruta
Puedes cambiar la ubicación usando la variable de entorno `LOG_FILE`:

```bash
# En Linux/macOS
export LOG_FILE=/ruta/personalizada/mi-app.log
./target/release/app-desktop-tauri

# En una sola línea
LOG_FILE=/ruta/personalizada/mi-app.log ./target/release/app-desktop-tauri
```

```powershell
# En Windows PowerShell
$env:LOG_FILE="C:\logs\mi-app.log"
.\target\release\app-desktop.exe
```

## 📋 Formato de los logs

Los logs incluyen timestamp, nivel y mensaje:

```
[2025-11-27 13:45:23.456] [INFO] === Iniciando aplicación Tauri ===
[2025-11-27 13:45:23.457] [INFO] Logs guardados en: /tmp/tauri-app.log
[2025-11-27 13:45:23.458] [INFO] Backend objetivo: http://localhost:3000/api
[2025-11-27 13:45:24.123] [DEBUG] Listando salas
[2025-11-27 13:45:24.234] [INFO] Listadas 3 salas
[2025-11-27 13:45:30.567] [INFO] Creando sala: Sala Conferencias
[2025-11-27 13:45:30.789] [INFO] Sala creada: Sala Conferencias (ID: abc-123)
```

## 📊 Niveles de log

- **INFO**: Operaciones normales (crear sala, listar, activar, etc.)
- **DEBUG**: Información de debugging (requests internos)
- **ERROR**: Errores en operaciones

## 🔍 Ver los logs

### En tiempo real
```bash
# Seguir los logs mientras la app está corriendo
tail -f /tmp/tauri-app.log
```

### Ver todo el archivo
```bash
cat /tmp/tauri-app.log
```

### Buscar errores
```bash
grep ERROR /tmp/tauri-app.log
```

### Ver logs de hoy
```bash
grep "$(date +%Y-%m-%d)" /tmp/tauri-app.log
```

## 🖥️ En la interfaz

La aplicación muestra la ruta del archivo de log en:
- **Consola JavaScript**: `console.log`
- **Banner inferior izquierdo**: Muestra la ruta completa del archivo de log

## 📝 Ejemplos de uso

### Desarrollo local
```bash
# Logs por defecto en /tmp
cargo run --release
```

### Producción con logs personalizados
```bash
# Logs en el directorio del usuario
LOG_FILE=~/logs/reservas-salas-$(date +%Y%m%d).log ./target/release/app-desktop-tauri
```

### Logs por día
```bash
# Crear un log diferente cada día
LOG_FILE="/var/log/reservas-salas-$(date +%Y-%m-%d).log" ./target/release/app-desktop-tauri
```

### Con servidor backend personalizado
```bash
# Configurar ambos: backend y logs
BACKEND_BASE_URL=http://api.example.com \
LOG_FILE=/var/log/app-desktop-tauri.log \
./target/release/app-desktop-tauri
```

## 🧹 Limpiar logs antiguos

```bash
# Eliminar logs más antiguos de 7 días
find /tmp -name "tauri-app.log*" -mtime +7 -delete

# Rotar logs manualmente
mv /tmp/tauri-app.log /tmp/tauri-app.log.$(date +%Y%m%d)
```

## 🐛 Debugging

Si necesitas más información de debugging:

1. **Abre DevTools** en la app (Cmd+Option+I en macOS)
2. **Revisa la consola de JavaScript** para logs del frontend
3. **Revisa el archivo de log** para logs del backend Rust

Los logs del backend (Rust) y frontend (JavaScript) están sincronizados:
- Frontend: `console.log()` en DevTools
- Backend: Archivo de log + stdout

## Variables de entorno disponibles

| Variable | Descripción | Default |
|----------|-------------|---------|
| `LOG_FILE` | Ruta del archivo de log | `/tmp/tauri-app.log` |
| `BACKEND_BASE_URL` | URL del servidor backend | `http://localhost:3000/api` |

## Ejemplo completo

```bash
#!/bin/bash
# Script de inicio con logging configurado

# Crear directorio de logs si no existe
mkdir -p ~/logs/reservas-salas

# Configurar variables
export LOG_FILE=~/logs/reservas-salas/app-$(date +%Y%m%d-%H%M%S).log
export BACKEND_BASE_URL=http://localhost:3000/api

# Ejecutar la app
/Users/davidlopez/Proyectos/ReservasSalas/target/release/app-desktop-tauri

# El banner en la UI mostrará la ruta del log
```

## 📱 Verificar que funciona

Después de arrancar la app:

1. **En la consola**:
   ```
   [2025-11-27 13:45:23.456] [INFO] === Iniciando aplicación Tauri ===
   [2025-11-27 13:45:23.457] [INFO] Logs guardados en: /tmp/tauri-app.log
   ```

2. **En la UI**: Banner inferior izquierdo mostrando la ruta

3. **En el archivo**:
   ```bash
   cat /tmp/tauri-app.log
   ```
