#include "ui/MainWindow.h"

#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QHeaderView>
#include <QFileDialog>
#include <QMessageBox>
#include <QStatusBar>
#include <QMenuBar>
#include <QMenu>
#include <QAction>
#include <QStyle>

namespace calyx::ui {

MainWindow::MainWindow(audio::IAudioEngine* engine, QWidget* parent)
    : QMainWindow(parent)
    , m_engine(engine)
    , m_player(std::make_unique<core::Player>(engine, this))
    , m_library(std::make_unique<core::LibraryManager>(this))
{
    buildUi();
    connectSignals();
}

// ═══════════════════════════════════════════════════════════════════════════
//  UI construction (pure code — no .ui file needed)
// ═══════════════════════════════════════════════════════════════════════════

void MainWindow::buildUi() {
    setWindowTitle("Calyx");
    resize(960, 640);

    // ── Menu bar ────────────────────────────────────────────────────────
    auto* fileMenu = menuBar()->addMenu(tr("&File"));
    auto* actScan  = fileMenu->addAction(tr("&Scan Folder…"));
    connect(actScan, &QAction::triggered, this, &MainWindow::onScanFolder);
    fileMenu->addSeparator();
    auto* actQuit = fileMenu->addAction(tr("&Quit"));
    connect(actQuit, &QAction::triggered, this, &QWidget::close);

    // ── Central widget ──────────────────────────────────────────────────
    auto* central = new QWidget(this);
    auto* mainLayout = new QVBoxLayout(central);

    // Search bar
    auto* searchLayout = new QHBoxLayout;
    m_searchBar = new QLineEdit(this);
    m_searchBar->setPlaceholderText(tr("Search library…"));
    m_btnScan = new QPushButton(tr("Scan Folder"), this);
    searchLayout->addWidget(m_searchBar, 1);
    searchLayout->addWidget(m_btnScan);
    mainLayout->addLayout(searchLayout);

    // Track table
    m_trackTable = new QTableWidget(0, 4, this);
    m_trackTable->setHorizontalHeaderLabels({tr("Title"), tr("Artist"), tr("Album"), tr("Duration")});
    m_trackTable->horizontalHeader()->setStretchLastSection(true);
    m_trackTable->horizontalHeader()->setSectionResizeMode(0, QHeaderView::Stretch);
    m_trackTable->setSelectionBehavior(QAbstractItemView::SelectRows);
    m_trackTable->setEditTriggers(QAbstractItemView::NoEditTriggers);
    m_trackTable->verticalHeader()->hide();
    mainLayout->addWidget(m_trackTable, 1);

    // Now-playing label
    m_lblNowPlaying = new QLabel(tr("No track loaded"), this);
    m_lblNowPlaying->setAlignment(Qt::AlignCenter);
    auto font = m_lblNowPlaying->font();
    font.setPointSize(font.pointSize() + 2);
    font.setBold(true);
    m_lblNowPlaying->setFont(font);
    mainLayout->addWidget(m_lblNowPlaying);

    // Seek slider + time label
    auto* seekLayout = new QHBoxLayout;
    m_seekSlider = new QSlider(Qt::Horizontal, this);
    m_seekSlider->setRange(0, 0);
    m_lblTime = new QLabel("0:00 / 0:00", this);
    seekLayout->addWidget(m_seekSlider, 1);
    seekLayout->addWidget(m_lblTime);
    mainLayout->addLayout(seekLayout);

    // Transport controls + volume
    auto* controlsLayout = new QHBoxLayout;
    m_btnPrev      = new QPushButton(this);
    m_btnPlayPause = new QPushButton(this);
    m_btnStop      = new QPushButton(this);
    m_btnNext      = new QPushButton(this);

    m_btnPrev->setIcon(style()->standardIcon(QStyle::SP_MediaSkipBackward));
    m_btnPlayPause->setIcon(style()->standardIcon(QStyle::SP_MediaPlay));
    m_btnStop->setIcon(style()->standardIcon(QStyle::SP_MediaStop));
    m_btnNext->setIcon(style()->standardIcon(QStyle::SP_MediaSkipForward));

    m_volumeSlider = new QSlider(Qt::Horizontal, this);
    m_volumeSlider->setRange(0, 100);
    m_volumeSlider->setValue(50);
    m_volumeSlider->setMaximumWidth(120);

    auto* volLabel = new QLabel(tr("Vol"), this);

    controlsLayout->addStretch();
    controlsLayout->addWidget(m_btnPrev);
    controlsLayout->addWidget(m_btnPlayPause);
    controlsLayout->addWidget(m_btnStop);
    controlsLayout->addWidget(m_btnNext);
    controlsLayout->addStretch();
    controlsLayout->addWidget(volLabel);
    controlsLayout->addWidget(m_volumeSlider);
    mainLayout->addLayout(controlsLayout);

    setCentralWidget(central);

    // Status bar
    m_statusLabel = new QLabel(tr("Ready"), this);
    statusBar()->addWidget(m_statusLabel);
}

// ═══════════════════════════════════════════════════════════════════════════
//  Signal wiring
// ═══════════════════════════════════════════════════════════════════════════

void MainWindow::connectSignals() {
    // Transport
    connect(m_btnPlayPause, &QPushButton::clicked, this, &MainWindow::onPlayPause);
    connect(m_btnStop,      &QPushButton::clicked, this, &MainWindow::onStop);
    connect(m_btnNext,      &QPushButton::clicked, this, &MainWindow::onNext);
    connect(m_btnPrev,      &QPushButton::clicked, this, &MainWindow::onPrevious);
    connect(m_volumeSlider, &QSlider::valueChanged, this, &MainWindow::onVolumeChanged);
    connect(m_seekSlider,   &QSlider::sliderReleased, this, [this]() { onSeek(m_seekSlider->value()); });

    // Search & library
    connect(m_searchBar, &QLineEdit::textChanged, this, &MainWindow::onSearchTextChanged);
    connect(m_btnScan,   &QPushButton::clicked,   this, &MainWindow::onScanFolder);
    connect(m_trackTable, &QTableWidget::cellDoubleClicked, this, &MainWindow::onTrackDoubleClicked);

    // Engine feedback
    connect(m_engine, &audio::IAudioEngine::stateChanged,    this, &MainWindow::onEngineStateChanged);
    connect(m_engine, &audio::IAudioEngine::positionChanged,  this, &MainWindow::onPositionChanged);
    connect(m_engine, &audio::IAudioEngine::durationChanged,  this, &MainWindow::onDurationChanged);

    // Library events
    connect(m_library.get(), &core::LibraryManager::scanFinished, this, &MainWindow::onScanFinished);
}

// ═══════════════════════════════════════════════════════════════════════════
//  Slots
// ═══════════════════════════════════════════════════════════════════════════

void MainWindow::onPlayPause() {
    if (m_engine->state() == audio::IAudioEngine::State::Playing)
        m_player->pause();
    else
        m_player->play();
}

void MainWindow::onStop()     { m_player->stop(); }
void MainWindow::onNext()     { m_player->next(); }
void MainWindow::onPrevious() { m_player->previous(); }

void MainWindow::onVolumeChanged(int value) {
    m_player->setVolume(value);
}

void MainWindow::onSeek(int value) {
    m_player->seek(std::chrono::milliseconds{value});
}

void MainWindow::onSearchTextChanged(const QString& text) {
    auto results = m_library->search(text);
    populateTrackTable(results);
}

void MainWindow::onScanFolder() {
    QString dir = QFileDialog::getExistingDirectory(this, tr("Select Music Folder"));
    if (dir.isEmpty()) return;
    m_statusLabel->setText(tr("Scanning…"));
    m_library->scanDirectory(dir);
}

void MainWindow::onTrackDoubleClicked(int row, int /*column*/) {
    auto tracks = m_library->search(m_searchBar->text());
    if (row >= 0 && row < tracks.size()) {
        m_player->playTrack(tracks[row]);
    }
}

void MainWindow::onEngineStateChanged(audio::IAudioEngine::State state) {
    if (state == audio::IAudioEngine::State::Playing)
        m_btnPlayPause->setIcon(style()->standardIcon(QStyle::SP_MediaPause));
    else
        m_btnPlayPause->setIcon(style()->standardIcon(QStyle::SP_MediaPlay));
}

void MainWindow::onPositionChanged(std::chrono::milliseconds pos) {
    if (!m_seekSlider->isSliderDown())
        m_seekSlider->setValue(static_cast<int>(pos.count()));

    auto fmt = [](std::chrono::milliseconds ms) {
        auto totalSec = std::chrono::duration_cast<std::chrono::seconds>(ms).count();
        return QString("%1:%2").arg(totalSec / 60).arg(totalSec % 60, 2, 10, QChar('0'));
    };
    auto dur = m_engine->duration();
    m_lblTime->setText(QString("%1 / %2").arg(fmt(pos), fmt(dur)));
}

void MainWindow::onDurationChanged(std::chrono::milliseconds dur) {
    m_seekSlider->setRange(0, static_cast<int>(dur.count()));
}

void MainWindow::onScanFinished(int count) {
    m_statusLabel->setText(tr("Library: %1 tracks").arg(count));
    populateTrackTable(m_library->allTracks());
}

// ═══════════════════════════════════════════════════════════════════════════
//  Helpers
// ═══════════════════════════════════════════════════════════════════════════

void MainWindow::populateTrackTable(const QVector<core::Track>& tracks) {
    m_trackTable->setRowCount(0);
    for (const auto& t : tracks) {
        int row = m_trackTable->rowCount();
        m_trackTable->insertRow(row);
        m_trackTable->setItem(row, 0, new QTableWidgetItem(t.title));
        m_trackTable->setItem(row, 1, new QTableWidgetItem(t.artist));
        m_trackTable->setItem(row, 2, new QTableWidgetItem(t.album));
        m_trackTable->setItem(row, 3, new QTableWidgetItem(t.durationFormatted()));
    }
}

} // namespace calyx::ui
