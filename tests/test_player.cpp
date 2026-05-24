#include <gtest/gtest.h>
#include "core/Player.h"
#include "audio/QtAudioEngine.h"

#include <QCoreApplication>

using namespace calyx::core;
using namespace calyx::audio;

class PlayerTest : public ::testing::Test {
protected:
    void SetUp() override {
        static int argc = 1;
        static char arg0[] = "test";
        static char* argv[] = {arg0};
        if (!QCoreApplication::instance())
            m_app = std::make_unique<QCoreApplication>(argc, argv);

        m_engine = std::make_unique<QtAudioEngine>();
        m_player = std::make_unique<Player>(m_engine.get());
    }

    std::unique_ptr<QCoreApplication> m_app;
    std::unique_ptr<QtAudioEngine>    m_engine;
    std::unique_ptr<Player>           m_player;
};

TEST_F(PlayerTest, InitiallyNoTrack) {
    EXPECT_FALSE(m_player->currentTrack().has_value());
    EXPECT_TRUE(m_player->queue().isEmpty());
}

TEST_F(PlayerTest, EnqueueAddsToQueue) {
    Track t;
    t.id = 1;
    t.title = "Test";
    t.filePath = QUrl::fromLocalFile("/test.mp3");

    m_player->enqueue(t);
    EXPECT_EQ(m_player->queue().size(), 1);
}

TEST_F(PlayerTest, ClearQueueResetsState) {
    Track t;
    t.id = 1;
    t.filePath = QUrl::fromLocalFile("/test.mp3");

    m_player->enqueue(t);
    m_player->clearQueue();
    EXPECT_TRUE(m_player->queue().isEmpty());
    EXPECT_FALSE(m_player->currentTrack().has_value());
}

TEST_F(PlayerTest, PlayTrackSetsCurrentTrack) {
    Track t;
    t.id = 42;
    t.title = "Hello";
    t.filePath = QUrl::fromLocalFile("/hello.mp3");

    m_player->playTrack(t);
    ASSERT_TRUE(m_player->currentTrack().has_value());
    EXPECT_EQ(m_player->currentTrack()->id, 42);
}
