# Calyx 🎵

> **Un reproductor de Apple Music elegante, rápido y sin distracciones para tu terminal.**

Calyx te permite escuchar toda tu música de Apple Music directamente desde la consola, con carátulas en arte ASCII, visualizador de audio en tiempo real, letras animadas sincronizadas y control total con el teclado.

---

## ✨ Características

- 🎧 **Audio de Alta Fidelidad**: Reproducción fluida en calidad AAC 256 kbps sin publicidad ni interrupciones.
- ⚡ **Reproducción Continua (Gapless)**: Cambios de canción instantáneos sin silencios incómodos entre pistas.
- 🎤 **Letras en Tiempo Real**: Las letras avanzan e iluminan cada sílaba al compás de la voz (estilo karaoke).
- 🈳 **Pronunciación Fonética (`y`)**: Transliteración automática a alfabeto latino (Romaji) para canciones en japonés o coreano.
- 🌈 **Visualizador de Espectro**: Animación de frecuencias retroiluminada de 48 pasos que responde a la música.
- 📁 **Biblioteca y Playlists**: Accede a tus listas guardadas, añade canciones (`m`), guarda álbumes (`f`) o crea listas nuevas al instante.
- 🌐 **Bilingüe (Español / English)**: Detecta el idioma de tu sistema y permite alternar en caliente con la tecla `L`.
- 💻 **Multiplataforma**: Funciona de forma idéntica en Windows, macOS y Linux.

---

## 🚀 Instalación y Uso Rápido

### 1. Descargar y compilar
Requiere tener [Rust instalado](https://rustup.rs/):

```sh
# Clonar el proyecto
git clone https://github.com/hector-horta/calyx.git
cd calyx

# Iniciar el reproductor
cargo run -p calyx-tui
```

> **Consejo regional**: Si quieres explorar las listas y novedades de un país específico, puedes pasar su código de 2 letras al arrancar (por ejemplo: `cargo run -p calyx-tui -- mx` para México, `es` para España o `ar` para Argentina).

---

## 🔑 Iniciar Sesión (Apple Music)

Para escuchar tus canciones completas, tus playlists y tu biblioteca personal:

1. Abre Calyx y presiona la tecla **`u`**.
2. Se abrirá automáticamente tu navegador habitual con la página oficial de Apple Music.
3. Inicia sesión normalmente. Calyx detectará tu cuenta y volverá a la terminal listo para reproducir.

*(Si alguna vez necesitas consultar opciones manuales para servidores sin entorno gráfico, presiona `?` dentro de Calyx).*

---

## ⌨️ Atajos de Teclado

### 🧭 Navegación y Búsqueda
| **Tecla** | **Acción** |
|:---:|:---|
| `/` | **Buscar** cualquier canción, artista o álbum en el catálogo |
| `1` a `5` | **Secciones principales**: `1` Recientes, `2` Novedades, `3` Radio, `4` Tu Biblioteca, `5` Playlists |
| `o` | **Ver Álbum**: Abre el álbum de la canción seleccionada (pulsar de nuevo para volver) |
| `Tab` | **Alternar vista**: Cambia entre la lista que estás explorando y tu Cola activa |
| `?` | **Pantalla de Ayuda** con guía de controles integrada |

### 🎵 Reproducción y Sonido
| **Tecla** | **Acción** |
|:---:|:---|
| `Enter` | **Reproducir** la canción seleccionada |
| `Espacio` | **Pausar / Reanudar** |
| `<` / `>` (o `,` / `.`) | **Pista Anterior / Siguiente** |
| `+` / `-` | **Subir / Bajar volumen** (+5% / -5%) |
| `s` | **Modo Aleatorio** (`🔀 Shuffle`) |
| `r` | **Modo Repetir** (`🔁 Todo` ➔ `🔂 Una canción` ➔ `Off`) |
| `p` | **Reproducir Todo**: Reproduce la lista o playlist completa |

### 📋 Gestión de Canciones y Listas
| **Tecla** | **Acción** |
|:---:|:---|
| `a` | **Encolar al final** de la reproducción |
| `n` | **Encolar a continuación** (*Play Next*) |
| `A` | **Encolar Toda la Lista** actual |
| `f` | **Favorito / Guardar**: Añade el álbum completo a tu Biblioteca |
| `m` | **Añadir a Playlist**: Abre el menú para guardarla en una lista existente o crear una nueva |
| `d` | **Quitar**: Quita de la Cola; en una Playlist abierta, presionar dos veces para borrarla |
| `c` | **Limpiar**: Vacía la cola activa o el historial |

### ⚙️ Preferencias
| **Tecla** | **Acción** |
|:---:|:---|
| `y` | **Pronunciación Fonética**: Alterna entre letra original, fonética o modo dual |
| `L` | **Cambiar Idioma**: Alterna al instante entre Español e Inglés |
| `u` | **Iniciar sesión** con tu cuenta de Apple Music |
| `q` | **Salir** de Calyx |

---

## 💡 Preguntas Frecuentes

- **¿Dónde se guarda mi sesión y configuración?**
  Calyx almacena tu sesión en las carpetas estándar de usuario de tu sistema (`%APPDATA%\calyx` en Windows, `~/.config/calyx` en Linux, `~/Library/Application Support/calyx` en macOS).
- **¿Calyx guarda música en mi disco?**
  Para que la música suene fluida y sin cortes, Calyx mantiene una pequeña caché temporal de audio en tu carpeta personal (`~/.cache/calyx/audio/`), la cual se autolimpia de forma automática para no exceder los 2 GB.
- **¿Puedo usarlo en modo portátil?**
  Sí, si defines la variable de entorno `CALYX_HOME=/ruta/carpeta`, Calyx guardará todos sus datos en esa ubicación sin tocar el resto de tu equipo.
