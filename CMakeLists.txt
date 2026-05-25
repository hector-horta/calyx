cmake_minimum_required(VERSION 3.21)

# ── vcpkg Integration ─────────────────────────────────────────────────────────
if(EXISTS "D:/dev/vcpkg/scripts/buildsystems/vcpkg.cmake")
    set(CMAKE_TOOLCHAIN_FILE "D:/dev/vcpkg/scripts/buildsystems/vcpkg.cmake" CACHE STRING "vcpkg toolchain" FORCE)
endif()

project(Calyx
    VERSION 0.1.0
    DESCRIPTION "High-performance music player"
    LANGUAGES CXX
)

# ── C++ standard ──────────────────────────────────────────────────────────────
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# ── Qt 6 ──────────────────────────────────────────────────────────────────────
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

find_package(Qt6 REQUIRED COMPONENTS Core Gui Qml Network Quick Widgets Sql Multimedia)

# ── vcpkg-managed dependencies ────────────────────────────────────────────────
find_package(nlohmann_json CONFIG REQUIRED)
find_package(GTest CONFIG REQUIRED)

# ── Compiler warnings ────────────────────────────────────────────────────────
if(MSVC)
    add_compile_options(/W4 /permissive-)
else()
    add_compile_options(-Wall -Wextra -Wpedantic)
endif()

# ── Source libraries ──────────────────────────────────────────────────────────
add_subdirectory(src/core)
add_subdirectory(src/audio)
add_subdirectory(src/network)

# ── Main executable ──────────────────────────────────────────────────────────
add_executable(calyx
    src/main.cpp
    src/ui/resources.qrc
)

target_link_libraries(calyx
    PRIVATE
        calyx_core
        calyx_audio
        calyx_network
        Qt6::Core
        Qt6::Gui
        Qt6::Qml
        Qt6::Quick
        Qt6::Widgets
)

# Import QML plugins dynamically (needed for static builds)
qt_import_qml_plugins(calyx)

# ── Tests ─────────────────────────────────────────────────────────────────────
enable_testing()
add_subdirectory(tests)
