# Calyx - Apple Music Native Terminal UI (TUI)

**Cliente de consola nativo en Rust para Apple Music con reproducción de audio de alta fidelidad, caché en disco gapless y visualizador analógico TrueColor.**

---

## 🔧 Arquitectura del Proyecto

### Workspace Cargo (`Cargo.toml`)

```
crates/
├── calyx-core          # Motor de audio nativo (Symphonia/Rodio), cliente API Apple Music, SQLite WAL,
│                       # parser TTML nativo y caché atómico en disco con política LRU.
└── calyx-tui           # Interfaz de consola (Ratatui), gapless lookahead prefetching,
                        # analizador de espectro de 48 pasos con fade in y letras animadas por carácter.
```

---

## 🚀 Instalación y Ejecución

```sh
# Clonar repositorio
git clone https://github.com/hector-horta/calyx.git
cd calyx

# Compilar y ejecutar
cargo run -p calyx-tui
```

---

## 🎵 Atajos de Teclado (TUI)

> 💡 **Inicio Rápido**: Presiona **`u`** al abrir Calyx para iniciar sesión en Apple Music y desbloquear audio en alta fidelidad (`[AAC 256k]`), tu biblioteca personal y tus playlists.

| **Tecla** | **Acción** |
|:---:|:---|
| `u` | **🔑 Iniciar Sesión / Login Apple Music** (*Recomendado*): Inicia sesión para desbloquear audio completo `[AAC 256k]`, Biblioteca y Playlists |
| `/` | **Buscar** canciones en el catálogo de Apple Music |
| `1` / `2` / `3` / `4` / `5` | **Secciones Apple Music** (`1 - Recientes`, `2 - Novedades`, `3 - Radio`, `4 - Biblioteca`, `5 - Playlists`) |
| `?` | **Ayuda** de controles y guía de configuración integrada |
| `p` | **Reproducir Lista / Play Playlist** (reproduce la lista o playlist completa) |
| `L` / `l` | **Cambiar Idioma** en caliente (Español ⇄ English) |
| `<` / `>` (o `,` / `.`) | **Pista Anterior / Siguiente** en la cola |
| `c` | **Limpiar Todo** (vacía la Cola activa o el historial de Recientes) |
| `y` | **Letras Fonéticas / Pronunciación** (cicla entre: `Dual [Kanji+Romaji]` ➔ `Romaji` ➔ `Original`) |
| `d` | **Quitar Pista** seleccionada (de la Cola o de Recientes) |
| `j` / `k` o `↑` / `↓` | **Navegar** arriba/abajo en la lista activa |
| `Enter` | **Reproducir** la canción seleccionada |
| `Espacio` | **Pausar / Reanudar** la reproducción |
| `a` | **Encolar al final** (*Add to queue*) |
| `n` | **Encolar a continuación** (*Play Next*) |
| `Tab` | **Alternar vista** entre *Catálogo* y *Cola de Reproducción* |
| `s` | **Alternar Shuffle** (Aleatorio `🔀 ON` / `OFF`) |
| `r` | **Alternar Repeat** (Repetición `🔁 Off` ➔ `🔁 All` ➔ `🔂 One`) |
| `+` / `-` | **Ajustar volumen** (+5% / -5%) |
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
- **🌈 Analizador de Espectro Analógico (48 Pasos de Fluidez)**:
  - 12 bandas de frecuencia de 4 caracteres de ancho (`████`) con ranuras sutiles de 1 espacio.
  - El avance temporal de la pista ilumina el espectro celda por celda con un barrido analógico suave mediante *color blending* (Dorado, Aqua y Verde esmeralda).
- **📊 Glifos de Estado Gráficos & Consola de Volumen**:
  - Iconos dinámicos de Play y Pausa de 3 filas de alto renderizados con halfblocks (`▀`) y autocrop inteligente de bounding box.
  - Medidor de volumen vertical alineado a la derecha (`VOL`, `85%`, ` ▂▄▆█`).

---

## 🔑 Autenticación (Apple Music Sign-In)

Calyx permite sincronizar tu biblioteca personal y reproducir el catálogo completo de Apple Music:

- **Modo Automático**: Presiona la tecla `u` en Calyx para abrir un inicio de sesión temporal y seguro en tu navegador predeterminado. Al autenticarte, Calyx sincroniza tu sesión automáticamente y el navegador se cierra de inmediato.
- **Modo Manual (Servidores Linux / SSH / Headless)**: Presiona `?` (Ayuda) directamente dentro de Calyx para consultar la **guía interactiva paso a paso** para configurar tu sesión mediante la variable de entorno `CALYX_MEDIA_USER_TOKEN` o el archivo de configuración `calyx_auth.json`.
- **Navegador Personalizado**: Si tu navegador está en una ruta no estándar o usas Flatpak, puedes especificarlo con la variable de entorno `CALYX_BROWSER_BIN`:
  ```bash
  export CALYX_BROWSER_BIN="/usr/bin/chromium"
  # O en Windows PowerShell:
  $env:CALYX_BROWSER_BIN = "C:\Ruta\A\Tu\chrome.exe"
  ```

---

## 🌐 Soporte Multi-idioma (i18n)

- **Detección Automática**: Calyx detecta el idioma configurado en tu sistema operativo (`LANG`, `LC_ALL`, `LC_MESSAGES`) al arrancar.
- **Idiomas Soportados**: Español (`Es`) e Inglés (`En`). Si tu sistema tiene un idioma distinto o no está disponible, Calyx iniciará por defecto en Inglés.
- **Alternar en Caliente (`L`)**: Puedes presionar la tecla `L` en cualquier momento para alternar al instante entre Español e Inglés sin reiniciar la aplicación ni alterar los atajos de teclado.
- **Persistencia**: La preferencia de idioma seleccionada se guarda automáticamente en `calyx_auth.json`.
