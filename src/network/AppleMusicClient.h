#pragma once

#include <QObject>
#include <QNetworkAccessManager>
#include <QNetworkReply>
#include <QString>

namespace calyx::network {

/// Asynchronous, non-blocking HTTP client for Apple Music API.
/// Uses QNetworkAccessManager internally.
class AppleMusicClient : public QObject {
    Q_OBJECT
public:
    explicit AppleMusicClient(QObject* parent = nullptr);
    ~AppleMusicClient() override = default;

    /// Fetches a test JSON payload.
    /// If developerToken is empty, it queries the public iTunes Search API.
    /// If developerToken is provided, it attempts to query the Apple Music charts API.
    void fetchTestJSON(const QString& developerToken = "");

signals:
    void requestStarted();
    void requestFinished(const QString& jsonResponse, bool success);

private:
    QNetworkAccessManager m_networkManager;
};

} // namespace calyx::network
