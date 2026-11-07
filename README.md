# Calyx - Apple Music Native Terminal UI (TUI)

**Cliente de consola nativo en Rust para Apple Music con reproducción de audio de alta fidelidad, desencriptación Widevine L3 / CBCS en memoria, caché en disco gapless y visualizador analógico TrueColor.**

---

## 🔧 Arquitectura del Proyecto

### Workspace Cargo (`Cargo.toml`)

```
crates/
├── calyx-core          # Motor de audio nativo (Symphonia/Rodio), cliente API Apple Music,
│                       # desencriptación CBCS/Widevine L3, SQLite WAL versionado,
│                       # parser TTML nativo y caché atómico en disco con política LRU.
└── calyx-tui           # Interfaz de consola (Ratatui), reproductor de dos cajas balanceado,
                        # lookahead prefetching gapless, espectro analógico de 48 pasos,
                        # avisos en reverse-video y letras sincronizadas animadas por carácter.
```

---

## 🚀 Instalación y Ejecución

```sh
# Clonar repositorio
git clone https://github.com/hector-horta/calyx.git
cd calyx

# Compilar y ejecutar (Storefront por defecto: "us")
cargo run -p calyx-tui

# Ejecutar con un catálogo regional específico (ej: México, España, Argentina)
cargo run -p calyx-tui -- mx
cargo run -p calyx-tui -- es
cargo run -p calyx-tui -- ar

# Consultar versión instalada
cargo run -p calyx-tui -- --version
```

---

## 🎵 Atajos de Teclado (TUI)

> 💡 **Inicio Rápido**: Presiona **`u`** al abrir Calyx para iniciar sesión en Apple Music y desbloquear audio en alta fidelidad (`[AAC 256k]`), tu biblioteca personal y tus playlists.

| **Tecla** | **Acción** |
|:---:|:---|
| `u` | **🔑 Iniciar Sesión / Login Apple Music** (*Recomendado*): Inicia sesión para desbloquear audio completo `[AAC 256k]`, Biblioteca y Playlists |
| `/` | **Buscar** canciones en el catálogo de Apple Music |
| `1` a `5` | **Secciones Apple Music** (`1 - Recientes`, `2 - Novedades`, `3 - Radio`, `4 - Biblioteca`, `5 - Playlists`) |
| `?` | **Ayuda** de controles y guía de configuración interactiva |
| `o` | **Álbum / Volver**: Abre el álbum de la canción seleccionada; pulsado dentro de un álbum regresa a la lista de origen |
| `f` | **Guardar Álbum**: Añade el álbum de la canción seleccionada a tu Biblioteca personal |
| `m` | **Agregar a Playlist**: Abre el selector para añadir la canción a una playlist existente o crear una nueva al vuelo |
| `p` | **Reproducir Todo**: Reproduce la lista o playlist activa completa |
| `Enter` | **Reproducir** la canción seleccionada |
| `Espacio` | **Pausar / Reanudar** la reproducción |
| `a` | **Encolar al final** (*Add to queue*) |
| `n` | **Encolar a continuación** (*Play Next*) |
| `A` | **Encolar Lista Entera**: Añade todas las canciones de la vista actual a la cola de reproducción |
| `d` | **Quitar**: De la Cola o Recientes. Dentro de una Playlist abierta, presionar `d` dos veces (con confirmación) elimina la canción |
| `c` | **Limpiar Todo**: Vacía la Cola activa o el historial de Recientes |
| `<` / `>` (o `,` / `.`) | **Pista Anterior / Siguiente** en la cola |
| `j` / `k` o `↑` / `↓` | **Navegar** arriba/abajo en la lista activa |
| `s` | **Alternar Shuffle** (Aleatorio `🔀 ON` / `OFF`) |
| `r` | **Alternar Repeat** (Repetición `🔁 Off` ➔ `🔁 All` ➔ `🔂 One`) |
| `+` / `-` | **Ajustar volumen** (+5% / -5%) |
| `y` | **Letras Fonéticas / Pronunciación**: Cicla entre `Dual [Kanji+Romaji]` ➔ `Romaji` ➔ `Original` |
| `L` / `l` | **Cambiar Idioma** en caliente (Español ⇄ English) |
| `Tab` | **Alternar vista** entre lista activa y *Cola de Reproducción* |
| `q` | **Salir** de Calyx |

---

## 🔐 Soporte de Audio, Caché y Streaming

- **Pipeline Nativo de Audio fMP4 & Widevine L3**: Soporte de decodificación continua AAC-LC 256kbps en memoria con `Symphonia` + `Rodio`, desencriptación local CBCS y metadatos de sincronización de audio.
- **⚡ Caché Atómico en Disco (`~/.cache/calyx/audio/`)**:
  - Almacena las pistas ya desencriptadas en disco con cuota máxima configurable de **2 GB**.
  - **Escritura Atómica Anti-Corrupción**: Escribe primero en `.part` y solo se consolida en `.bin` al completar el stream al 100%. Los saltos (*Skip*) nunca corrompen el caché.
  - **Política de Desalojo LRU**: Cada reproducción renueva la fecha de acceso (`mtime`/`touch`). Si el tamaño excede la cuota, el recolector de basura en segundo plano elimina las pistas más antiguas hasta recuperar un 25% de espacio libre.
- **🎧 Lookahead Prefetching de la Pista $n+1$ (Gapless Playback)**:
  - Mientras la canción actual $n$ está sonando, Calyx descarga y desencripta automáticamente en segundo plano la siguiente pista de la cola ($n+1$).
  - Al cambiar de canción o terminar la pista, la siguiente arranca en **0.05 segundos** con el tag `[AAC 256k | Caché]`, eliminando por completo los baches de silencio entre pistas.
- **🎤 Letras Sincronizadas TTML con Relleno por Carácter (Fade-In RGB)**:
  - Parser nativo TTML de Apple Music con timestamps de alta precisión, con fallback automático a LRCLIB.
  - Resaltado letra por letra en **24-bit TrueColor**: la vocal o consonante cantada se ilumina progresivamente con interpolación de color continua desde gris tenue hasta blanco brillante (`Color::White` BOLD) al compás del canto.
  - **🈳 Pronunciación Fonética y Transliteración (`y`)**: Motor nativo offline que translitera caracteres asiáticos a Romaji/alfabeto latino (Japonés Kanji/Kana mediante KAKASI y Coreano Hangul por descomposición silábica). Presiona `y` para ciclar entre `Dual` (original + fonética simultánea), `Romaji` (solo fonética) o `Desactivado`.
- **🌈 Analizador de Espectro y Reproductor Equilibrado de Dos Cajas**:
  - Diseño de interfaz centrado con portada, controles compactos e integración del indicador de volumen en la misma línea de mandos.
  - 12 bandas de frecuencia de 4 caracteres de ancho (`████`) con ranuras sutiles de 1 espacio.
  - El avance temporal de la pista ilumina el espectro celda por celda con un barrido analógico suave mediante *color blending* (Dorado, Aqua y Verde esmeralda).
- **🔔 Avisos Reverse-Video & Indicadores de Actividad en Tiempo Real**:
  - Notificaciones de alta visibilidad en video invertido para confirmaciones críticas (agregar a biblioteca, listas o confirmaciones de borrado).
  - Spinner animado de actividad y contador $N/M$ en la barra de estado mientras se cargan listas o se sincronizan letras.

---

## 🔑 Autenticación y Ubicaciones Multiplataforma

Calyx permite sincronizar tu biblioteca personal y reproducir el catálogo completo de Apple Music:

- **Modo Automático Multi-Navegador**: Presiona la tecla `u` en Calyx. Detecta dinámicamente tu navegador predeterminado y extrae la sesión sin requerir pasos manuales:
  * **Chromium y derivados** *(Chrome, Edge, Brave, Arc, Opera, Vivaldi)*: Vía puerto temporal de depuración CDP.
  * **Firefox y derivados** *(Firefox, LibreWolf, Zen Browser, Waterfox, Floorp)*: Vía lectura de snapshot local de `cookies.sqlite`.
  * **Safari y WebKit** *(macOS Safari, Orion, GNOME Web)*: Vía lectura nativa de `Cookies.binarycookies`.
  > [!NOTE]
  > **Usuarios de macOS (Safari):** Al presionar `u`, macOS te solicitará permiso para que la Terminal lea la sesión de Safari. Haz clic en **"Permitir"** en el diálogo del sistema para que Calyx capture el token de forma automática.
- **Modo Manual (Servidores Linux / SSH / Headless)**: Presiona `?` (Ayuda) directamente dentro de Calyx para consultar la **guía interactiva paso a paso** para configurar tu sesión mediante la variable de entorno `CALYX_MEDIA_USER_TOKEN` o el archivo `calyx_auth.json`.
- **Navegador Personalizado**: Si tu navegador está en una ruta no estándar o usas Flatpak, puedes especificarlo con la variable de entorno `CALYX_BROWSER_BIN`:
  ```bash
  export CALYX_BROWSER_BIN="/usr/bin/chromium"
  # O en Windows PowerShell:
  $env:CALYX_BROWSER_BIN = "C:\Ruta\A\Tu\chrome.exe"
  ```
- **Rutas del Sistema según Plataforma**:
  * **Configuración y Sesión (`calyx_auth.json`)**:
    - Windows: `%APPDATA%\calyx\calyx_auth.json`
    - Linux: `~/.config/calyx/calyx_auth.json` (XDG)
    - macOS: `~/Library/Application Support/calyx/calyx_auth.json`
  * **Base de Datos SQLite (`calyx_cache.db`)**: En el directorio de datos local de cada SO.
  * **Caché de Audio Descifrado**: `~/.cache/calyx/audio/`
  * **Instalaciones Portables**: Define la variable de entorno `CALYX_HOME` para almacenar toda la configuración, base de datos y caché en un directorio unificado (ej: pendrive o carpeta del proyecto).

---

## 🌐 Soporte Multi-idioma (i18n)

- **Detección Automática**: Calyx detecta el idioma configurado en tu sistema operativo (`LANG`, `LC_ALL`, `LC_MESSAGES`) al arrancar.
- **Idiomas Soportados**: Español (`Es`) e Inglés (`En`). Si tu sistema tiene un idioma distinto o no está disponible, Calyx iniciará por defecto en Inglés.
- **Alternar en Caliente (`L`)**: Puedes presionar la tecla `L` en cualquier momento para alternar al instante entre Español e Inglés sin reiniciar la aplicación ni alterar los atajos de teclado.
- **Persistencia**: La preferencia de idioma seleccionada se guarda automáticamente en `calyx_auth.json`.
