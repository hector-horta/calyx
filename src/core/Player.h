#pragma once

#include "core/Track.h"
#include "audio/IAudioEngine.h"

#include <QObject>
#include <QQueue>
#include <memory>
#include <optional>

namespace calyx::core {

/// High-level player that manages the play queue and delegates to an IAudioEngine.
class Player : public QObject {
    Q_OBJECT
public:
    explicit Player(audio::IAudioEngine* engine, QObject* parent = nullptr);
    ~Player() override = default;

    // ── Queue management ────────────────────────────────────────────────
    void enqueue(const Track& track);
    void clearQueue();
    [[nodiscard]] const QList<Track>& queue() const;

    // ── Playback ────────────────────────────────────────────────────────
    void playTrack(const Track& track);
    void play();
    void pause();
    void stop();
    void next();
    void previous();

    void seek(std::chrono::milliseconds pos);
    void setVolume(int volume);

    // ── Queries ─────────────────────────────────────────────────────────
    [[nodiscard]] std::optional<Track> currentTrack() const;
    [[nodiscard]] audio::IAudioEngine::State state() const;

signals:
    void currentTrackChanged(const calyx::core::Track& track);
    void queueChanged();

private:
    audio::IAudioEngine* m_engine;  // non-owning; lifetime managed externally
    QList<Track>         m_queue;
    int                  m_currentIndex{-1};

    void loadCurrent();
};

} // namespace calyx::core
