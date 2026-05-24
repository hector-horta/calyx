#include <gtest/gtest.h>
#include "core/LibraryManager.h"

#include <QCoreApplication>
#include <QTemporaryDir>
#include <QFile>

using namespace calyx::core;

class LibraryManagerTest : public ::testing::Test {
protected:
    void SetUp() override {
        // QCoreApplication is needed for Qt event loop in some Qt internals.
        static int argc = 1;
        static char arg0[] = "test";
        static char* argv[] = {arg0};
        if (!QCoreApplication::instance())
            m_app = std::make_unique<QCoreApplication>(argc, argv);
    }

    std::unique_ptr<QCoreApplication> m_app;
};

TEST_F(LibraryManagerTest, InitiallyEmpty) {
    LibraryManager lm;
    EXPECT_EQ(lm.trackCount(), 0);
    EXPECT_TRUE(lm.allTracks().isEmpty());
}

TEST_F(LibraryManagerTest, ScanFindsAudioFiles) {
    QTemporaryDir tmpDir;
    ASSERT_TRUE(tmpDir.isValid());

    // Create dummy audio files (content doesn't matter for scanning).
    for (const auto& name : {"song1.mp3", "song2.flac", "readme.txt"}) {
        QFile f(tmpDir.filePath(name));
        f.open(QIODevice::WriteOnly);
        f.write("dummy");
        f.close();
    }

    LibraryManager lm;
    lm.scanDirectory(tmpDir.path());

    // Should find .mp3 and .flac but not .txt
    EXPECT_EQ(lm.trackCount(), 2);
}

TEST_F(LibraryManagerTest, SearchByTitle) {
    QTemporaryDir tmpDir;
    ASSERT_TRUE(tmpDir.isValid());

    QFile f(tmpDir.filePath("MyGreatSong.mp3"));
    f.open(QIODevice::WriteOnly);
    f.write("dummy");
    f.close();

    LibraryManager lm;
    lm.scanDirectory(tmpDir.path());

    auto results = lm.search("great");
    EXPECT_EQ(results.size(), 1);

    auto empty = lm.search("nonexistent");
    EXPECT_TRUE(empty.isEmpty());
}
