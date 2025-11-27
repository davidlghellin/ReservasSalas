# Guía de Distribución - Reservas Salas

Guía completa para compilar y distribuir la aplicación en diferentes plataformas.

## 📦 Plataformas soportadas

| Plataforma | Estado | Documentación |
|------------|--------|---------------|
| 🍎 **macOS** | ✅ Listo | Este archivo |
| 🐧 **Linux** | ✅ CI/CD | [GitHub Actions](../../.github/workflows/build-desktop.yml) |
| 🪟 **Windows** | ✅ CI/CD | [GitHub Actions](../../.github/workflows/build-desktop.yml) |
| 🤖 **Android** | ✅ Listo | [ANDROID.md](ANDROID.md) |
| 🌐 **Web (SPA)** | ✅ Listo | [WEB.md](WEB.md) |
| 🌐 **Web (SSR)** | ✅ Listo | `crates/app-web` |
| 🍎 **iOS** | 🚧 Tauri v2.1+ | Próximamente |

---

## 🚀 Compilación rápida

### macOS (nativo)

```bash
cd crates/app-desktop/src-tauri
cargo build --release

# Ejecutar:
../../../target/release/app-desktop
```

### Linux, Windows y Android

**GitHub Actions (automático al crear tag)**

```bash
# 1. Hacer cambios y commit
git add .
git commit -m "feat: nueva funcionalidad"
git push

# 2. Crear tag para compilar
git tag v0.1.0
git push origin v0.1.0

# Esto automáticamente:
# - Compila binarios para Linux, macOS (x64 + ARM64), Windows
# - Crea instaladores (DMG, DEB, MSI)
# - Crea un GitHub Release con todos los archivos
```

**O ejecutar manualmente desde GitHub Actions:**
1. Ir a: Actions → "Build Desktop Apps" → "Run workflow"
2. Los binarios estarán disponibles en Artifacts

### Android APK (manual)

Para compilar Android APK desde GitHub Actions:

1. Ir a: Actions → "Build Desktop Apps" → "Run workflow"
2. Seleccionar la rama
3. Ejecutar
4. El APK estará en Artifacts: `app-desktop-android`

Ver [ANDROID.md](ANDROID.md) para compilación local (requiere Android SDK/NDK)

### Web

**Ya existe app-web con SSR:**
```bash
cargo run --bin server
# http://localhost:3000
```

**O crear SPA (ver WEB.md):**
```bash
cd crates/app-desktop-web
python3 -m http.server 8080
# http://localhost:8080
```

---

## 🏗️ Crear instaladores

### macOS (.dmg)

```bash
cd crates/app-desktop/src-tauri

# Instalar tauri-cli si no lo tienes
cargo install tauri-cli

# Crear bundle
cargo tauri build

# DMG en: target/release/bundle/dmg/
```

### Linux (.deb, .AppImage)

```bash
cd crates/app-desktop/src-tauri
cargo tauri build

# Archivos en:
# - target/release/bundle/deb/app-desktop.deb
# - target/release/bundle/appimage/app-desktop.AppImage
```

### Windows (.msi)

```powershell
cd crates\app-desktop\src-tauri
cargo tauri build

# MSI en: target\release\bundle\msi\
```

---

## ⚙️ Variables de entorno

### Para todas las plataformas

```bash
# Cambiar URL del backend
export BACKEND_BASE_URL=https://api.example.com

# Cambiar ruta de logs
export LOG_FILE=/var/log/reservas-salas.log

# Ejecutar
./target/release/app-desktop
```

### En Windows

```powershell
$env:BACKEND_BASE_URL="https://api.example.com"
$env:LOG_FILE="C:\logs\reservas-salas.log"
.\target\release\app-desktop.exe
```

---

## 📊 Tamaños de binarios

| Plataforma | Debug | Release | Comprimido |
|------------|-------|---------|------------|
| macOS | ~80MB | ~10MB | ~3MB |
| Linux | ~75MB | ~9MB | ~2.8MB |
| Windows | ~85MB | ~11MB | ~3.5MB |
| Android APK | - | ~15MB | ~15MB |
| Web (SPA) | - | ~500KB | ~150KB |

---

## 🎯 GitHub Actions (CI/CD)

El proyecto incluye workflows automatizados:

### `.github/workflows/build-desktop.yml`

Compila automáticamente para **Linux, macOS (x64 + ARM64) y Windows**:

```yaml
# Se ejecuta cuando:
# - Creas un tag: git tag v0.1.0 && git push origin v0.1.0
# - Manualmente desde GitHub Actions
```

**Workflow automático al crear tag:**
1. Compila binarios para todas las plataformas
2. Crea instaladores (DMG, DEB, MSI)
3. Crea GitHub Release automáticamente
4. Sube todos los binarios e instaladores al release

### Descargar binarios

**Desde un Release:**
1. Ir a: `https://github.com/TU_USUARIO/ReservasSalas/releases`
2. Descargar los instaladores o binarios

**Desde Actions (ejecución manual):**
1. Ir a: `https://github.com/TU_USUARIO/ReservasSalas/actions`
2. Seleccionar el workflow "Build Desktop Apps"
3. Descargar los artifacts:
   - `app-desktop-linux-x64`
   - `app-desktop-macos-x64`
   - `app-desktop-macos-arm64`
   - `app-desktop-windows-x64`
   - `app-desktop-android` (si se ejecutó manualmente)

---

## 🔖 Crear release

### 1. Incrementar versión

Editar `crates/app-desktop/src-tauri/tauri.conf.json`:

```json
{
  "version": "1.2.0"
}
```

Y `Cargo.toml`:

```toml
[package]
version = "1.2.0"
```

### 2. Crear tag

```bash
git tag v1.2.0
git push origin v1.2.0
```

### 3. Descargar builds

Los instaladores se generan automáticamente en GitHub Actions:
- macOS: `.dmg`
- Linux: `.deb`, `.AppImage`
- Windows: `.msi`

---

## 📱 Distribución móvil

### Android

#### Google Play Store

```bash
# Crear App Bundle (AAB)
cd crates/app-desktop/src-tauri/gen/android
./gradlew bundleRelease

# Subir a: https://play.google.com/console
```

#### F-Droid

```bash
# APK firmado
cargo tauri android build --release

# Enviar a F-Droid: https://f-droid.org/
```

### iOS (Próximamente)

Tauri v2.1+ soportará iOS. La configuración será similar a Android.

---

## 🌐 Distribución web

### Opción 1: Servidor propio

```bash
# Compilar backend
cargo build --release --bin server

# Ejecutar en producción
BACKEND_BASE_URL=https://api.example.com \
LOG_FILE=/var/log/app.log \
./target/release/server
```

### Opción 2: Netlify/Vercel (SPA)

```bash
cd crates/app-desktop-spa
npm run build

# Subir carpeta dist/
```

### Opción 3: Docker

```bash
docker build -t reservas-salas .
docker run -p 3000:3000 reservas-salas
```

---

## 🔐 Firmar aplicaciones

### macOS (Notarización)

```bash
# Requiere Apple Developer Account

# Firmar
codesign --force --deep --sign "Developer ID Application: TU NOMBRE" \
  target/release/bundle/macos/app-desktop.app

# Notarizar
xcrun notarytool submit target/release/bundle/dmg/app-desktop.dmg \
  --apple-id tu@email.com \
  --team-id TEAM_ID \
  --password APP_SPECIFIC_PASSWORD
```

### Windows (Certificado)

```powershell
# Requiere certificado de code signing

signtool sign /f certificado.pfx /p PASSWORD /t http://timestamp.digicert.com `
  target\release\bundle\msi\app-desktop.msi
```

### Android (Keystore)

Ver [ANDROID.md](ANDROID.md#-firmar-apk-para-producción)

---

## 📋 Checklist de release

- [ ] Incrementar versión en `tauri.conf.json` y `Cargo.toml`
- [ ] Actualizar CHANGELOG.md
- [ ] Probar en todas las plataformas
- [ ] Crear tag de git
- [ ] Esperar a que GitHub Actions compile
- [ ] Descargar y probar los binarios
- [ ] Firmar aplicaciones (macOS, Windows)
- [ ] Crear release en GitHub con notas
- [ ] Subir APK/AAB a Play Store (si aplica)
- [ ] Actualizar documentación

---

## 🐛 Solución de problemas

### Build falla en Linux

```bash
# Instalar dependencias
sudo apt-get install libwebkit2gtk-4.1-dev build-essential libssl-dev
```

### Build falla en Windows

```powershell
# Instalar Visual Studio Build Tools
# https://visualstudio.microsoft.com/downloads/
```

### APK no instala

```bash
# Verificar firma
jarsigner -verify -verbose -certs app-release.apk
```

### App no se conecta al backend

```bash
# Verificar variable de entorno
echo $BACKEND_BASE_URL

# Configurar si es necesario
export BACKEND_BASE_URL=http://localhost:3000/api
```

---

## 📚 Documentación adicional

- [README.md](README.md) - Información general
- [ANDROID.md](ANDROID.md) - Compilar para Android
- [WEB.md](WEB.md) - Versión web/SPA
- [LOGGING.md](LOGGING.md) - Sistema de logs

---

## 🔗 Links útiles

- [Tauri Docs](https://v2.tauri.app/)
- [GitHub Actions](https://docs.github.com/en/actions)
- [Google Play Console](https://play.google.com/console)
- [Apple Developer](https://developer.apple.com/)
