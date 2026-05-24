find_package(GTest CONFIG REQUIRED)

# ── Track unit tests ─────────────────────────────────────────────────────────
add_executable(test_track test_track.cpp)
target_link_libraries(test_track
    PRIVATE
        calyx_core
        calyx_audio
        GTest::gtest_main
        Qt6::Core
)
add_test(NAME TrackTests COMMAND test_track)

# ── LibraryManager unit tests ────────────────────────────────────────────────
add_executable(test_library_manager test_library_manager.cpp)
target_link_libraries(test_library_manager
    PRIVATE
        calyx_core
        calyx_audio
        GTest::gtest_main
        Qt6::Core
)
add_test(NAME LibraryManagerTests COMMAND test_library_manager)

# ── Player unit tests ────────────────────────────────────────────────────────
add_executable(test_player test_player.cpp)
target_link_libraries(test_player
    PRIVATE
        calyx_core
        calyx_audio
        GTest::gtest_main
        Qt6::Core
        Qt6::Multimedia
)
add_test(NAME PlayerTests COMMAND test_player)
