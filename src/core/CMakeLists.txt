add_library(calyx_core STATIC
    Track.h
    Player.h        Player.cpp
    LibraryManager.h LibraryManager.cpp
)

target_include_directories(calyx_core PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/..)

target_link_libraries(calyx_core
    PUBLIC
        Qt6::Core
        Qt6::Sql
        nlohmann_json::nlohmann_json
    PRIVATE
        calyx_audio
)
