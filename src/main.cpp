#include "core/MusicPlayer.h"
#include "ui/CoverImageProvider.h"
#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QUrl>

int main(int argc, char* argv[]) {
    QGuiApplication app(argc, argv);
    QGuiApplication::setApplicationName("Calyx");
    QGuiApplication::setApplicationVersion("0.1.0");
    QGuiApplication::setOrganizationName("Calyx");

    // Register C++ MusicPlayer to QML
    qmlRegisterType<calyx::core::MusicPlayer>("Calyx", 1, 0, "MusicPlayer");

    QQmlApplicationEngine engine;
    engine.addImageProvider(QStringLiteral("covers"), new CoverImageProvider());

    // Load the main QML interface from the compiled resources
    const QUrl url(u"qrc:/main.qml"_qs);
    QObject::connect(
        &engine,
        &QQmlApplicationEngine::objectCreationFailed,
        &app,
        []() { QCoreApplication::exit(-1); },
        Qt::QueuedConnection);
    engine.load(url);

    return app.exec();
}
