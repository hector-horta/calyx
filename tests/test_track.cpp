#include <gtest/gtest.h>
#include "core/Track.h"

using namespace calyx::core;

TEST(TrackTest, DefaultTrackIsInvalid) {
    Track t;
    EXPECT_FALSE(t.isValid());
}

TEST(TrackTest, TrackWithFilePathIsValid) {
    Track t;
    t.filePath = QUrl::fromLocalFile("/music/song.mp3");
    EXPECT_TRUE(t.isValid());
}

TEST(TrackTest, DurationFormatted) {
    Track t;
    t.duration = std::chrono::milliseconds{185000}; // 3:05
    EXPECT_EQ(t.durationFormatted(), "3:05");
}

TEST(TrackTest, DurationFormattedZero) {
    Track t;
    t.duration = std::chrono::milliseconds{0};
    EXPECT_EQ(t.durationFormatted(), "0:00");
}

TEST(TrackTest, Equality) {
    Track a, b;
    a.id = 1; a.filePath = QUrl::fromLocalFile("/a.mp3");
    b.id = 1; b.filePath = QUrl::fromLocalFile("/a.mp3");
    EXPECT_EQ(a, b);

    b.id = 2;
    EXPECT_NE(a, b);
}
