#include "audio/QtAudioEngine.h"

namespace calyx::audio {

QtAudioEngine::QtAudioEngine(QObject* parent)
    : IAudioEngine(parent)
    , m_player(std::make_unique<QMediaPlayer>())
    , m_audioOutput(std::make_unique<QAudioOutput>())
{
    m_player->setAudioOutput(m_audioOutput.get());
    m_audioOutput->setVolume(0.5f); // 50 %
    connectSignals();
}

// ── Playback control ────────────────────────────────────────────────────────

void QtAudioEngine::play()  { m_player->play();  }
void QtAudioEngine::pause() { m_player->pause(); }
void QtAudioEngine::stop()  { m_player->stop();  }

void QtAudioEngine::setSource(const QUrl& url) {
    m_player->setSource(url);
}

void QtAudioEngine::setPosition(std::chrono::milliseconds pos) {
    m_player->setPosition(pos.count());
}

void QtAudioEngine::setVolume(int vol) {
    // Map 0-100 int to 0.0-1.0 float for Qt.
    float normalized = std::clamp(vol, 0, 100) / 100.0f;
    m_audioOutput->setVolume(normalized);
}

// ── Queries ─────────────────────────────────────────────────────────────────

IAudioEngine::State QtAudioEngine::state() const {
    switch (m_player->playbackState()) {
        case QMediaPlayer::PlayingState: return State::Playing;
        case QMediaPlayer::PausedState:  return State::Paused;
        default:                         return State::Stopped;
    }
}

std::chrono::milliseconds QtAudioEngine::position() const {
    return std::chrono::milliseconds{m_player->position()};
}

std::chrono::milliseconds QtAudioEngine::duration() const {
    return std::chrono::milliseconds{m_player->duration()};
}

int QtAudioEngine::volume() const {
    return static_cast<int>(m_audioOutput->volume() * 100.0f);
}

// ── Internal ────────────────────────────────────────────────────────────────

void QtAudioEngine::connectSignals() {
    connect(m_player.get(), &QMediaPlayer::playbackStateChanged,
            this, [this](QMediaPlayer::PlaybackState s) {
                State mapped = State::Stopped;
                if (s == QMediaPlayer::PlayingState) mapped = State::Playing;
                else if (s == QMediaPlayer::PausedState) mapped = State::Paused;
                emit stateChanged(mapped);
            });

    connect(m_player.get(), &QMediaPlayer::positionChanged,
            this, [this](qint64 pos) {
                emit positionChanged(std::chrono::milliseconds{pos});
            });

    connect(m_player.get(), &QMediaPlayer::durationChanged,
            this, [this](qint64 dur) {
                emit durationChanged(std::chrono::milliseconds{dur});
            });

    connect(m_player.get(), &QMediaPlayer::errorOccurred,
            this, [this](QMediaPlayer::Error /*err*/, const QString& msg) {
                emit errorOccurred(msg);
            });
}

} // namespace calyx::audio
