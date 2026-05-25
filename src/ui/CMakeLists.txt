qt_add_qml_module(calyx_ui
    URI Calyx.UI
    VERSION 1.0
    QML_FILES
        main.qml
        Sidebar.qml
        PlaybackBar.qml
        MainContent.qml
        ApiView.qml
)

target_include_directories(calyx_ui PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/..)

target_link_libraries(calyx_ui
    PUBLIC
        Qt6::Core
        Qt6::Qml
        Qt6::Quick
        calyx_core
)
