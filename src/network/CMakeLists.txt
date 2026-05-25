add_library(calyx_network STATIC
    AppleMusicClient.h
    AppleMusicClient.cpp
)

target_include_directories(calyx_network PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/..)

target_link_libraries(calyx_network
    PUBLIC
        Qt6::Core
        Qt6::Network
)
