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

- **Buffer Progresivo fMP4 (Gapless Streaming)**: Inicio de reproducción tras los primeros 2 fragmentos `moof + mdat` (~160 KB) con encolado en segundo plano (`rodio::Sink::append`) y desencriptación Widevine L3 CENC/CBCS al vuelo.
- **Resolución Multi-Candidato (`amp-api` + `universalLibraryId`)**: Soporte nativo para canciones de Biblioteca/Playlists (`i.xxxx`) y fallback automático en el catálogo de streaming de Apple Music.
- **Letras Sincronizadas en Vivo (`🎤 Letra en Vivo`)**: Panel lateral tipo karaoke con parser `.lrc`, auto-scroll y resaltado en tiempo real mediante reloj monotónico.
- **Pipeline Asíncrono No Bloqueante**: Descarga de audio, portadas TrueColor (`▀`) y letras en hilos dedicados (`tokio::spawn` + canal `BgEvent`) manteniendo la TUI a 60 FPS.

