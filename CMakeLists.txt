cmake_minimum_required(VERSION 3.21)
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

find_package(Qt6 REQUIRED COMPONENTS Core Widgets Multimedia Network Sql)

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
add_subdirectory(src/ui)

# ── Main executable ──────────────────────────────────────────────────────────
add_executable(calyx src/main.cpp)

target_link_libraries(calyx
    PRIVATE
        calyx_core
        calyx_audio
        calyx_ui
        Qt6::Core
        Qt6::Widgets
)

# ── Tests ─────────────────────────────────────────────────────────────────────
enable_testing()
add_subdirectory(tests)

# ── Resources (placeholder) ──────────────────────────────────────────────────
# qt_add_resources(calyx "resources" PREFIX "/" FILES resources/calyx.qrc)
