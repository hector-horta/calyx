# Especificación Técnica: Cliente Apple Music Nativo (Apollo)

## 1. Stack Tecnológico Seleccionado
- **Lenguaje:** C++20 (Uso intensivo de Smart Pointers y tipado fuerte).
- **Framework UI:** Qt 6.7.3 (QML + Qt Quick Controls 2).
- **Compilador:** MSVC 2019/2022 64-bit (Windows) / GCC (Linux).
- **Sistema de Build:** CMake.
- **Motor de Audio:** libVLC (vía wrapper nativo en C++).
- **DRM/Playback:** Inicialmente vía QtWebEngine (Headless) para el stream de Apple Music.

## 2. Arquitectura de Carpetas
El agente debe organizar el código de la siguiente manera:
- `/src/core`: Lógica de negocios, manejo de tokens de Apple Music, SQLite.
- `/src/audio`: Implementación del player con libVLC.
- `/src/network`: Cliente REST usando `QNetworkAccessManager`.
- `/src/ui`: Archivos `.qml` y recursos de diseño.
- `/assets`: Iconos, fuentes personalizadas y estilos.

## 3. Directrices de Performance (Crítico)
- **Zero Blocking:** Ninguna llamada de red o de base de datos puede ocurrir en el hilo principal (Main Thread).
- **QML Optimization:** Usar `Instantiator` o `ListView` con delegados livianos para las listas de canciones.
- **Caching:** Implementar un sistema de caché de imágenes asíncrono para las portadas de álbumes.
- **GPU Rendering:** La UI debe aprovechar el Scene Graph de Qt para animaciones fluidas a 60 FPS.

## 4. Primeras Tareas para el Agente
1. Generar el `CMakeLists.txt` inicial vinculando Qt6 (Core, Gui, Qml, Network, Quick).
2. Crear la clase base `MusicPlayer` en C++ y exponerla a QML usando `qmlRegisterType`.
3. Crear un esqueleto de UI en QML que tenga un Sidebar y una sección de contenido principal.
4. Implementar un "Hello World" que traiga un JSON de prueba de la API de Apple Music.
