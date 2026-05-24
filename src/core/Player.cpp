#include "core/Player.h"

namespace calyx::core {

Player::Player(audio::IAudioEngine* engine, QObject* parent)
    : QObject(parent)
    , m_engine(engine)
{
    Q_ASSERT(m_engine);

    // Auto-advance to the next track when the current one finishes.
    connect(m_engine, &audio::IAudioEngine::stateChanged,
            this, [this](audio::IAudioEngine::State s) {
                if (s == audio::IAudioEngine::State::Stopped &&
                    m_currentIndex >= 0 &&
                    m_currentIndex < m_queue.size() - 1) {
                    next();
                }
            });
}

// ── Queue management ────────────────────────────────────────────────────────

void Player::enqueue(const Track& track) {
    m_queue.append(track);
    emit queueChanged();
}

void Player::clearQueue() {
    m_queue.clear();
    m_currentIndex = -1;
    m_engine->stop();
    emit queueChanged();
}

const QList<Track>& Player::queue() const { return m_queue; }

// ── Playback ────────────────────────────────────────────────────────────────

void Player::playTrack(const Track& track) {
    // Replace queue with single track and play.
    m_queue.clear();
    m_queue.append(track);
    m_currentIndex = 0;
    loadCurrent();
    m_engine->play();
    emit currentTrackChanged(track);
    emit queueChanged();
}

void Player::play()  { m_engine->play();  }
void Player::pause() { m_engine->pause(); }
void Player::stop()  { m_engine->stop();  }

void Player::next() {
    if (m_currentIndex < m_queue.size() - 1) {
        ++m_currentIndex;
        loadCurrent();
        m_engine->play();
        emit currentTrackChanged(m_queue[m_currentIndex]);
    }
}

void Player::previous() {
    if (m_currentIndex > 0) {
        --m_currentIndex;
        loadCurrent();
        m_engine->play();
        emit currentTrackChanged(m_queue[m_currentIndex]);
    }
}

void Player::seek(std::chrono::milliseconds pos) { m_engine->setPosition(pos); }
void Player::setVolume(int vol) { m_engine->setVolume(vol); }

// ── Queries ─────────────────────────────────────────────────────────────────

std::optional<Track> Player::currentTrack() const {
    if (m_currentIndex >= 0 && m_currentIndex < m_queue.size())
        return m_queue[m_currentIndex];
    return std::nullopt;
}

audio::IAudioEngine::State Player::state() const {
    return m_engine->state();
}

// ── Internal ────────────────────────────────────────────────────────────────

void Player::loadCurrent() {
    if (m_currentIndex >= 0 && m_currentIndex < m_queue.size())
        m_engine->setSource(m_queue[m_currentIndex].filePath);
}

} // namespace calyx::core
