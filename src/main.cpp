#include "audio/QtAudioEngine.h"
#include "ui/MainWindow.h"

#include <QApplication>
#include <memory>

int main(int argc, char* argv[]) {
    QApplication app(argc, argv);
    QApplication::setApplicationName("Calyx");
    QApplication::setApplicationVersion("0.1.0");
    QApplication::setOrganizationName("Calyx");

    // Create the audio engine (owned by the application scope).
    auto engine = std::make_unique<calyx::audio::QtAudioEngine>();

    // Create and show the main window.
    calyx::ui::MainWindow mainWindow(engine.get());
    mainWindow.show();

    return app.exec();
}
