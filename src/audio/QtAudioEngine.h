#pragma once

#include "audio/IAudioEngine.h"

#include <QMediaPlayer>
#include <QAudioOutput>
#include <memory>

namespace calyx::audio {

/// Concrete audio engine backed by Qt 6 Multimedia.
class QtAudioEngine : public IAudioEngine {
    Q_OBJECT
public:
    explicit QtAudioEngine(QObject* parent = nullptr);
    ~QtAudioEngine() override = default;

    // ── IAudioEngine interface ──────────────────────────────────────────
    void play()  override;
    void pause() override;
    void stop()  override;
    void setSource(const QUrl& url) override;
    void setPosition(std::chrono::milliseconds pos) override;
    void setVolume(int volume) override;

    [[nodiscard]] State state() const override;
    [[nodiscard]] std::chrono::milliseconds position() const override;
    [[nodiscard]] std::chrono::milliseconds duration() const override;
    [[nodiscard]] int volume() const override;

private:
    void connectSignals();

    std::unique_ptr<QMediaPlayer>  m_player;
    std::unique_ptr<QAudioOutput>  m_audioOutput;
};

} // namespace calyx::audio
