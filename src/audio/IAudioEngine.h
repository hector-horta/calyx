#pragma once

#include <QObject>
#include <QUrl>
#include <chrono>

namespace calyx::audio {

/// Abstract interface for audio playback engines.
/// Allows swapping Qt Multimedia for BASS or another backend later.
class IAudioEngine : public QObject {
    Q_OBJECT
public:
    enum class State { Stopped, Playing, Paused };
    Q_ENUM(State)

    explicit IAudioEngine(QObject* parent = nullptr) : QObject(parent) {}
    ~IAudioEngine() override = default;

    // ── Playback control ────────────────────────────────────────────────
    virtual void play()  = 0;
    virtual void pause() = 0;
    virtual void stop()  = 0;

    /// Load a new source to play.
    virtual void setSource(const QUrl& url) = 0;

    /// Seek to a position in the track.
    virtual void setPosition(std::chrono::milliseconds pos) = 0;

    /// Set volume in range [0, 100].
    virtual void setVolume(int volume) = 0;

    // ── Queries ─────────────────────────────────────────────────────────
    [[nodiscard]] virtual State state() const = 0;
    [[nodiscard]] virtual std::chrono::milliseconds position() const = 0;
    [[nodiscard]] virtual std::chrono::milliseconds duration() const = 0;
    [[nodiscard]] virtual int volume() const = 0;

signals:
    void stateChanged(IAudioEngine::State newState);
    void positionChanged(std::chrono::milliseconds position);
    void durationChanged(std::chrono::milliseconds duration);
    void errorOccurred(const QString& message);
};

} // namespace calyx::audio
