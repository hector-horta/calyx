add_library(calyx_audio STATIC
    IAudioEngine.h
    QtAudioEngine.h  QtAudioEngine.cpp
)

target_include_directories(calyx_audio PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/..)

target_link_libraries(calyx_audio
    PUBLIC
        Qt6::Core
        Qt6::Multimedia
)
