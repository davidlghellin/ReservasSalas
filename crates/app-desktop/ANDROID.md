# Compilar APK para Android

Guía para crear una aplicación Android (APK) a partir de la app de escritorio usando Tauri.

## 🔧 Requisitos previos

### 1. Android Studio y SDK

**Descargar e instalar:**
- [Android Studio](https://developer.android.com/studio)

**Después de instalar, abrir Android Studio y:**
1. Ir a `Settings/Preferences` → `Appearance & Behavior` → `System Settings` → `Android SDK`
2. En la pestaña `SDK Platforms`, instalar:
   - Android 13.0 (API 33) o superior
3. En la pestaña `SDK Tools`, instalar:
   - ✅ Android SDK Build-Tools
   - ✅ NDK (Side by side)
   - ✅ CMake
   - ✅ Android SDK Command-line Tools

### 2. Configurar variables de entorno

**macOS/Linux** - Añadir a `~/.zshrc` o `~/.bashrc`:

```bash
# Android SDK
export ANDROID_HOME=$HOME/Library/Android/sdk

# Android NDK
export NDK_HOME=$ANDROID_HOME/ndk/$(ls -1 $ANDROID_HOME/ndk | tail -1)

# Path
export PATH=$PATH:$ANDROID_HOME/emulator
export PATH=$PATH:$ANDROID_HOME/platform-tools
export PATH=$PATH:$ANDROID_HOME/cmdline-tools/latest/bin
```

**Aplicar cambios:**
```bash
source ~/.zshrc  # o source ~/.bashrc
```

**Verificar instalación:**
```bash
echo $ANDROID_HOME
echo $NDK_HOME
which adb
```

### 3. Instalar targets de Rust para Android

```bash
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add i686-linux-android
rustup target add x86_64-linux-android
```

### 4. Instalar Tauri CLI (si no lo tienes)

```bash
cargo install tauri-cli --version "^2.0.0"
```

## 📱 Inicializar proyecto Android

```bash
cd /Users/davidlopez/Proyectos/ReservasSalas/crates/app-desktop/src-tauri

# Inicializar Android (solo la primera vez)
cargo tauri android init
```

Esto creará:
- `gen/android/` - Proyecto Android nativo
- Configuración de Gradle
- Manifests de Android

## 🏗️ Compilar APK

### Desarrollo (Debug)

```bash
cd src-tauri

# Compilar APK de desarrollo
cargo tauri android build

# O con modo específico
cargo tauri android build --target aarch64
```

### Producción (Release)

```bash
# APK firmado para producción
cargo tauri android build --release
```

## 📦 Ubicación del APK

Después de compilar, el APK estará en:

```
src-tauri/gen/android/app/build/outputs/apk/
├── debug/
│   └── app-debug.apk
└── release/
    └── app-release-unsigned.apk
```

## 📲 Instalar en dispositivo

### En emulador Android

```bash
# Listar emuladores disponibles
emulator -list-avds

# Iniciar emulador
emulator -avd Pixel_5_API_33

# Instalar APK
adb install src-tauri/gen/android/app/build/outputs/apk/debug/app-debug.apk
```

### En dispositivo físico

1. **Habilitar modo desarrollador** en el dispositivo:
   - Configuración → Acerca del teléfono
   - Tocar 7 veces en "Número de compilación"

2. **Habilitar depuración USB**:
   - Configuración → Opciones de desarrollador
   - Activar "Depuración USB"

3. **Conectar dispositivo y verificar**:
```bash
adb devices
```

4. **Instalar APK**:
```bash
adb install src-tauri/gen/android/app/build/outputs/apk/debug/app-debug.apk
```

## 🔑 Firmar APK para producción

### 1. Crear keystore

```bash
keytool -genkey -v -keystore ~/reservas-salas.keystore \
  -alias reservas-salas \
  -keyalg RSA \
  -keysize 2048 \
  -validity 10000
```

### 2. Configurar firma en `build.gradle`

Editar `src-tauri/gen/android/app/build.gradle`:

```gradle
android {
    signingConfigs {
        release {
            storeFile file(System.getenv("KEYSTORE_PATH") ?: "${System.properties['user.home']}/reservas-salas.keystore")
            storePassword System.getenv("KEYSTORE_PASSWORD")
            keyAlias System.getenv("KEY_ALIAS")
            keyPassword System.getenv("KEY_PASSWORD")
        }
    }

    buildTypes {
        release {
            signingConfig signingConfigs.release
            minifyEnabled true
            proguardFiles getDefaultProguardFile('proguard-android-optimize.txt'), 'proguard-rules.pro'
        }
    }
}
```

### 3. Compilar con firma

```bash
export KEYSTORE_PATH=~/reservas-salas.keystore
export KEYSTORE_PASSWORD=tu_password
export KEY_ALIAS=reservas-salas
export KEY_PASSWORD=tu_password

cargo tauri android build --release
```

## 🎨 Personalizar la app Android

### Icono de la aplicación

Colocar iconos en:
```
src-tauri/gen/android/app/src/main/res/
├── mipmap-mdpi/ic_launcher.png (48x48)
├── mipmap-hdpi/ic_launcher.png (72x72)
├── mipmap-xhdpi/ic_launcher.png (96x96)
├── mipmap-xxhdpi/ic_launcher.png (144x144)
└── mipmap-xxxhdpi/ic_launcher.png (192x192)
```

### Permisos adicionales

Editar `src-tauri/gen/android/app/src/main/AndroidManifest.xml`:

```xml
<manifest>
    <!-- Permisos adicionales -->
    <uses-permission android:name="android.permission.INTERNET" />
    <uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />

    <application>
        ...
    </application>
</manifest>
```

### Nombre de la app

Editar `src-tauri/gen/android/app/src/main/res/values/strings.xml`:

```xml
<resources>
    <string name="app_name">Reservas Salas</string>
</resources>
```

## 🧪 Ejecutar en modo desarrollo

```bash
# Ejecutar en emulador o dispositivo conectado
cargo tauri android dev
```

Esto:
- Compila la app
- La instala en el dispositivo
- Abre DevTools para debugging
- Recarga automáticamente al cambiar código

## 📊 Optimizaciones para Android

### 1. Reducir tamaño del APK

En `tauri.conf.json`:

```json
{
  "bundle": {
    "android": {
      "minSdkVersion": 24,
      "allowedLibArchs": ["arm64-v8a", "armeabi-v7a"]
    }
  }
}
```

### 2. Configurar ProGuard

Crear `src-tauri/gen/android/app/proguard-rules.pro`:

```proguard
-keep class com.reservas.salas.** { *; }
-dontwarn com.reservas.salas.**
```

## ❗ Problemas comunes

### Error: "NDK not found"

```bash
# Verificar que NDK_HOME esté configurado
echo $NDK_HOME

# Si está vacío, reinstalar NDK desde Android Studio
# SDK Manager → SDK Tools → NDK (Side by side)
```

### Error: "ANDROID_HOME not set"

```bash
# Añadir a ~/.zshrc o ~/.bashrc
export ANDROID_HOME=$HOME/Library/Android/sdk
source ~/.zshrc
```

### Error: "No connected devices"

```bash
# Verificar conexión
adb devices

# Si está vacío, revisar:
# 1. Depuración USB habilitada
# 2. Cable USB conectado correctamente
# 3. Autorización en el dispositivo
```

### Compilación muy lenta

```bash
# Compilar solo para tu arquitectura
cargo tauri android build --target aarch64-linux-android
```

## 📱 Subir a Google Play Store

### 1. Crear App Bundle (AAB)

```bash
cd src-tauri/gen/android
./gradlew bundleRelease

# El AAB estará en:
# app/build/outputs/bundle/release/app-release.aab
```

### 2. Subir a Play Console

1. Ir a [Google Play Console](https://play.google.com/console)
2. Crear nueva aplicación
3. Subir el archivo `.aab`
4. Completar información de la tienda
5. Publicar

## 🔗 Referencias

- [Tauri Mobile Docs](https://v2.tauri.app/distribute/mobile/)
- [Android Developer Guide](https://developer.android.com/guide)
- [Cargo NDK](https://github.com/bbqsrc/cargo-ndk)

## 📋 Checklist de publicación

- [ ] Keystore creado y guardado de forma segura
- [ ] APK firmado con versión release
- [ ] Probado en múltiples dispositivos
- [ ] Permisos mínimos necesarios
- [ ] Iconos de todos los tamaños
- [ ] Screenshots para Play Store
- [ ] Política de privacidad publicada
- [ ] Descripción y capturas de pantalla
- [ ] Versión incrementada en `tauri.conf.json`
