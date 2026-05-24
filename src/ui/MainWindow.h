#pragma once

#include "core/Player.h"
#include "core/LibraryManager.h"
#include "audio/IAudioEngine.h"

#include <QMainWindow>
#include <QSlider>
#include <QPushButton>
#include <QLabel>
#include <QTableWidget>
#include <QLineEdit>
#include <memory>

namespace calyx::ui {

/// Main application window — wires together the library view, search,
/// player controls, and status information.
class MainWindow : public QMainWindow {
    Q_OBJECT
public:
    explicit MainWindow(audio::IAudioEngine* engine, QWidget* parent = nullptr);
    ~MainWindow() override = default;

private slots:
    void onPlayPause();
    void onStop();
    void onNext();
    void onPrevious();
    void onVolumeChanged(int value);
    void onSeek(int value);
    void onSearchTextChanged(const QString& text);
    void onScanFolder();
    void onTrackDoubleClicked(int row, int column);

    void onEngineStateChanged(audio::IAudioEngine::State state);
    void onPositionChanged(std::chrono::milliseconds pos);
    void onDurationChanged(std::chrono::milliseconds dur);
    void onScanFinished(int count);

private:
    void buildUi();
    void connectSignals();
    void populateTrackTable(const QVector<core::Track>& tracks);

    // ── Owned domain objects ────────────────────────────────────────────
    audio::IAudioEngine*                   m_engine;   // non-owning
    std::unique_ptr<core::Player>          m_player;
    std::unique_ptr<core::LibraryManager>  m_library;

    // ── Widgets ─────────────────────────────────────────────────────────
    QTableWidget* m_trackTable   = nullptr;
    QLineEdit*    m_searchBar    = nullptr;
    QPushButton*  m_btnPlayPause = nullptr;
    QPushButton*  m_btnStop      = nullptr;
    QPushButton*  m_btnPrev      = nullptr;
    QPushButton*  m_btnNext      = nullptr;
    QPushButton*  m_btnScan      = nullptr;
    QSlider*      m_volumeSlider = nullptr;
    QSlider*      m_seekSlider   = nullptr;
    QLabel*       m_lblNowPlaying = nullptr;
    QLabel*       m_lblTime      = nullptr;
    QLabel*       m_statusLabel  = nullptr;
};

} // namespace calyx::ui
