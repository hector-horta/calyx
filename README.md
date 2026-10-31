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
| `j` / `k` o `↑` / `↓` | **Navegar** arriba/abajo en la lista activa |
| `Enter` | **Reproducir** la canción seleccionada |
| `Espacio` | **Pausar / Reanudar** la reproducción |
| `a` | **Encolar al final** (*Add to queue*) |
| `n` | **Encolar a continuación** (*Play Next*) |
| `Tab` | **Alternar vista** entre *Catálogo* y *Cola de Reproducción* |
| `d` | **Eliminar** la canción seleccionada de la cola (en vista Cola) |
| `s` | **Alternar Shuffle** (Aleatorio `🔀 ON` / `OFF`) |
| `r` | **Alternar Repeat** (Repetición `🔁 Off` ➔ `🔁 All` ➔ `🔂 One`) |
| `+` / `-` | **Ajustar volumen** (+5% / -5%) |
| `u` | **Login Apple Music** (Abre navegador temporal para extraer `media-user-token`) |
| `t` | **Sincronizar Developer Token** de Apple Music |
| `q` | **Salir** de Calyx |

---

## 🧪 Pruebas Unitarias

```sh
# Ejecutar tests en el workspace completo
cargo test
```

---

## 🔐 Soporte de Audio y Streaming

- **Playback Híbrido**: Reproducción de pistas completas en streaming mediante desencriptación CBCS subsample con Widevine L3, y fallback automático a previews oficiales AAC de 30s.
- **Renderizado de Cover Art**: Mapeo matricial de portadas en la terminal mediante bloques Unicode de medio bloque (`▀`) y paleta RGB de 24-bit.
- **Cola en Memoria y Auto-Advance**: Reproducción secuencial ininterrumpida con cola reactiva y soporte para repetición y mezcla aleatoria.
