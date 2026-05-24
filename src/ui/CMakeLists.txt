add_library(calyx_ui STATIC
    MainWindow.h  MainWindow.cpp
)

target_include_directories(calyx_ui PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/..)

target_link_libraries(calyx_ui
    PUBLIC
        Qt6::Core
        Qt6::Widgets
        Qt6::Multimedia
        calyx_core
        calyx_audio
)
