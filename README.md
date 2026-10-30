# Calyx - Apple Music Client for Tauri Console UI

**Tauri v2 + Rust (Widevine L3 CENC/CBCS Support)**

---

## 🔧 Arquitectura del Proyecto

### Workspace Cargo (`Cargo.toml`)

```toml
[workspace]
resolver = "2"
members = [
    "crates/calyx-core",
    "crates/calyx-tui",
]

dependencies = {
    # Common dependencies
    calyx-core = { path = "./crates/calyx-core" }
}
```

### Estructura de Crates

```
crates/
├── calyx-core          # Lógica principal (APIs, decodificación, Widevine)
│   ├── src/
│   │   ├── audio.rs    # Decodificador de audio y gestión de streams
│   │   ├── widevine.rs # Soporte para CENC/CBCS (L3)
│   │   └── playback.rs # Control de reproducción nativo
│   └── Cargo.toml
└── calyx-tui           # Interfaz de consola (TUI)
    ├── src/
    │   ├── main.rs      # Entrada principal (Tauri + TUI)
    │   ├── ui.rs        # Renderizado y eventos
    │   └── controls.rs  # Manejo de teclas
    └── Cargo.toml
```

---

## 🚀 Instalación y Ejecución

### Dependencias

```sh
# Clonar repositorio
git clone https://github.com/calyx-dev/calyx.git
cd calyx

# Instalar dependencias (Rust + Tauri)
cargo install --workspace --features "widevine"  # Para soporte L3
```

### Ejecutar en Consola

```sh
cargo run -p calyx-tui
```
*(Usa `Ctrl+C` para detener la ejecución)*

---

## 🎵 Controladores de Consola (TUI)

| **Tecla**       | **Acción**                          |
|-----------------|-------------------------------------|
| `/`             | Buscar canciones                    |
| `j`/`k`/`↑`/`↓` | Navegar lista de reproducción        |
| `Enter`         | Reproducir canción actual           |
| `Espacio`       | Pausar/Reanudar                     |
| `+`/`-`         | Aumentar/Reducir volumen            |
| `q`             | Salir del cliente                   |

---

## 🧪 Pruebas

```sh
# Ejecutar tests en el workspace completo
cargo test --workspace --release

# Para pruebas específicas de decodificación (ejemplo)
cd crates/calyx-core && cargo test -p calyx-core
```

---

## 🔐 Soporte para Widevine L3 CENC/CBCS

Calyx incluye soporte nativo para:
- **Decodificación de streams cifrados** (L3).
- **Autenticación y desbloqueo** con CBCS.
- Integración con Tauri para manejo seguro en consola.

*(Requiere configuración adicional en `tauri.conf.json` para permisos de audio.)*

---

## 📜 Configuración Recomendada

1. **Tauri Config (`tauri.conf.json`)**
   ```json
   {
     "tauri": {
       "build": {
         "cargoArgs": ["--features", "widevine"],
         "targetDir": "./target"
       }
     },
     "appId": "com.calyx.dev",
     "version": "0.1.0"
   }
   ```

2. **Variables de Entorno (Opcional)**
   ```sh
   export CALYX_API_KEY="tu_api_key_here"  # Para autenticación en API externa
   ```

---

## 📌 Notas Adicionales

- **Soporte Multiplataforma**: Funciona en Linux/macOS con consolas TUI (como `alacritty` o `wezterm`).
- **Actualizaciones**: Usa `cargo update --workspace` para mantener dependencias al día.
- **Logs**: Verifica errores en terminal o directorio de logs (`./target/debug/`).

---
**Contribuciones**: Abre un *Pull Request* con mejoras en la arquitectura TUI o soporte para nuevas funciones (ej: playlist personalizable).