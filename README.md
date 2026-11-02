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
| `p` | **Reproducir toda la lista / Playlist actual** en segundo plano |
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
- **Letras Sincronizadas en Vivo (`🎤 Letra en Vivo`)**: Panel lateral tipo karaoke con auto-scroll y resaltado en tiempo real guiado por reloj monotónico (`Instant`).

