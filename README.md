# Calyx - Apple Music Native Terminal UI (TUI)

**Cliente de consola ultra-rápido en Rust para Apple Music con soporte Widevine L3 CENC/CBCS**

---

## 🔧 Arquitectura del Proyecto

### Workspace Cargo (`Cargo.toml`)

```
crates/
├── calyx-core          # Motor de audio nativo (Symphonia/Rodio), APIs de Apple Music, SQLite WAL, Widevine L3
└── calyx-tui           # Interfaz de consola (Ratatui), colas de reproducción y render de Cover Art ASCII/Unicode
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
| `?` | **Ayuda** de controles y atajos integrados |
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
| `u` | **Login Apple Music** (Abre navegador temporal para extraer `media-user-token`) |
| `t` | **Sincronizar Developer Token** de Apple Music |
| `q` | **Salir** de Calyx |

---

## 🔐 Soporte de Audio y Streaming

- **Streaming Completo fMP4 Widevine L3 (`cbcs` / `cenc`)**: Descarga y desencriptación en memoria de pistas completas AAC-LC 256kbps (`28:ctrp256`), preservando los punteros `sidx` y `tfhd.base_data_offset` para decodificación continua en `Symphonia` + `Rodio`.
- **Relinking Oficial de Catálogo (`?include=catalog` estilo Cider)**: Sincronización de Biblioteca y Playlists resolviendo el `salableAdamId` activo en tu *storefront*, con fallback automático contra `amp-api.music.apple.com` y `universalLibraryId` (`i.xxxx`).
- **Paquete Unificado Audio + Letra (`tokio::join!`)**: Descarga concurrente del stream de audio y letras sincronizadas (`.lrc`) con timeouts de red estrictos, entregados en un único evento atómico (`BgEvent::AudioReady`) junto al render asíncrono de portadas TrueColor (`▀`).
- **Analizador de Espectro Winamp + Avance de Canción**: Visualizador de 32 bandas de frecuencia pre-calculado fuera del hilo de audio (`skill-rust-audio-tauri`). Las barras verticales Unicode (`▁▂▃▄▅▆▇█`) bailan en tiempo real mientras actúan como barra de progreso: las frecuencias a la izquierda del avance actual brillan encendidas (verde esmeralda ➔ cian eléctrico ➔ dorado) y las restantes permanecen en tono apagado.
- **Panel de Letras en Vivo (Modo Karaoke)**: Resaltado dinámico palabra por palabra (`karaoke_parts`) guiado por reloj monotónico (`Instant`), con detección de interludios y cuenta regresiva instrumental animada (`● ○ ○` ➔ `● ● ●`) estilo Apple Music.
- **Reproductor De-cluttered**: Panel inferior minimalista y limpio con cabecera unificada (`Título — Artista • Álbum`), reloj integrado con tiempo restante negativo `[01:24 / 03:45 (-02:21)]`, volumen y próxima pista en cola, eliminando atajos redundantes.


---

## 🔑 Autenticación (Apple Music Sign-In)

Calyx permite reproducir el catálogo completo de Apple Music y tu biblioteca personal mediante dos modalidades:

### 1. Inicio de Sesión Automático (Interactivo)
Al presionar la tecla `u` en la interfaz TUI:
- Calyx detecta automáticamente cualquier navegador basado en Chromium instalado en tu sistema operativo:
  - **Windows**: Microsoft Edge, Google Chrome, Brave Browser.
  - **Linux**: Google Chrome, Chromium, Brave Browser, Microsoft Edge (en `/usr/bin/`).
  - **macOS**: Google Chrome, Brave Browser, Microsoft Edge (en `/Applications/`).
- Abre una ventana efímera y segura mediante Chrome DevTools Protocol (CDP) apuntando a `music.apple.com`.
- Al iniciar sesión con tu Apple ID, Calyx intercepta automáticamente la cookie `media-user-token`, la persiste en `calyx_auth.json` y cierra el navegador inmediatamente.
- **Ruta personalizada de navegador:** Si usas Flatpak u otra instalación personalizada, define la variable de entorno:
  ```bash
  export CALYX_BROWSER_BIN="/usr/bin/chromium"
  # O en Windows PowerShell:
  $env:CALYX_BROWSER_BIN = "C:\Ruta\A\Tu\chrome.exe"
  ```

### 2. Modo Manual (Servidores Linux, Docker y entornos sin navegador gráfico)
Si ejecutas Calyx en un servidor headless, máquina virtual o contenedor sin interfaz gráfica:
1. Inicia sesión en [music.apple.com](https://music.apple.com) desde el navegador de tu computadora.
2. Abre las Herramientas de Desarrollador (`F12` o `Ctrl+Shift+I` / `Cmd+Option+I`).
3. Ve a **Application** (o **Almacenamiento**) ➔ **Cookies** ➔ `https://music.apple.com`.
4. Copia el valor de la cookie llamada `media-user-token`.
5. Proporciónala a Calyx por cualquiera de estos dos métodos:
   - **Variable de Entorno:**
     ```bash
     export CALYX_MEDIA_USER_TOKEN="tu_media_user_token_aqui"
     ```
   - **Archivo de Configuración (`calyx_auth.json`):**
     Crea o edita `calyx_auth.json` en la raíz del proyecto:
     ```json
     {
       "storefront": "us",
       "developer_token": null,
       "media_user_token": "tu_media_user_token_aqui",
       "wvd_path": null,
       "language": "Es"
     }
     ```

---

## 🌐 Soporte Multi-idioma (i18n)

- **Detección Automática**: Calyx detecta el idioma configurado en tu sistema operativo (`LANG`, `LC_ALL`, `LC_MESSAGES`) al arrancar.
- **Idiomas Soportados**: Español (`Es`) e Inglés (`En`). Si tu sistema tiene un idioma distinto o no está disponible, Calyx iniciará por defecto en Inglés.
- **Alternar en Caliente (`L`)**: Puedes presionar la tecla `L` en cualquier momento para alternar al instante entre Español e Inglés sin reiniciar la aplicación ni alterar los atajos de teclado.
- **Persistencia**: La preferencia de idioma seleccionada se guarda automáticamente en `calyx_auth.json`.
