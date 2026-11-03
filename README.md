# Calyx - Apple Music Native Terminal UI (TUI)

**Cliente de consola nativo en Rust para Apple Music con reproducción de audio de alta fidelidad**

---

## 🔧 Arquitectura del Proyecto

### Workspace Cargo (`Cargo.toml`)

```
crates/
├── calyx-core          # Motor de audio nativo (Symphonia/Rodio), cliente API Apple Music, SQLite WAL
└── calyx-tui           # Interfaz de consola (Ratatui), colas de reproducción, letras en vivo y visualizador
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

| **Tecla** | **Acción** |
|:---:|:---|
| `/` | **Buscar** canciones en el catálogo de Apple Music |
| `1` / `2` / `3` / `4` / `5` | **Secciones Apple Music** (`1 - Recientes`, `2 - Novedades`, `3 - Radio`, `4 - Biblioteca`, `5 - Playlists`) |
| `?` | **Ayuda** de controles y guía de configuración integrada |
| `p` | **Reproducir Lista / Play Playlist** en segundo plano |
| `L` / `l` | **Cambiar Idioma** en caliente (Español ⇄ English) |
| `<` / `>` (o `,` / `.`) | **Pista Anterior / Siguiente** en la cola |
| `c` | **Limpiar Todo** (vacía la Cola activa o el historial SQLite de Recientes) |
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
| `u` | **Login Apple Music** (Abre navegador temporal para sincronizar sesión) |
| `t` | **Sincronizar Developer Token** de Apple Music |
| `q` | **Salir** de Calyx |

---

## 🔐 Soporte de Audio y Streaming

- **Pipeline Nativo de Audio fMP4**: Soporte de decodificación continua AAC-LC 256kbps en memoria con `Symphonia` + `Rodio`, preservando los metadatos de sincronización de audio.
- **Relinking Oficial de Catálogo (`?include=catalog` estilo Cider)**: Sincronización de Biblioteca y Playlists resolviendo el `salableAdamId` activo en tu *storefront*, con fallback automático contra `amp-api.music.apple.com` y `universalLibraryId` (`i.xxxx`).
- **Paquete Unificado Audio + Letra (`tokio::join!`)**: Descarga concurrente del stream de audio y letras sincronizadas (`.lrc`) con timeouts de red estrictos, entregados en un único evento atómico (`BgEvent::AudioReady`) junto al render asíncrono de portadas TrueColor (`▀`).
- **Analizador de Espectro Winamp + Avance de Canción**: Visualizador de 32 bandas de frecuencia pre-calculado fuera del hilo de audio (`skill-rust-audio-tauri`). Las barras verticales Unicode (`▁▂▃▄▅▆▇█`) bailan en tiempo real mientras actúan como barra de progreso: las frecuencias a la izquierda del avance actual brillan encendidas (verde esmeralda ➔ cian eléctrico ➔ dorado) y las restantes permanecen en tono apagado.
- **Panel de Letras en Vivo (Modo Karaoke)**: Resaltado dinámico palabra por palabra (`karaoke_parts`) guiado por reloj monotónico (`Instant`), con detección de interludios y cuenta regresiva instrumental animada (`● ○ ○` ➔ `● ● ●`) estilo Apple Music.
- **Reproductor De-cluttered**: Panel inferior minimalista y limpio con cabecera unificada (`Título — Artista • Álbum`), reloj integrado con tiempo restante negativo `[01:24 / 03:45 (-02:21)]`, volumen y próxima pista en cola, eliminando atajos redundantes.

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
