#include "AppleMusicClient.h"
#include <QNetworkRequest>
#include <QJsonDocument>
#include <QJsonObject>
#include <QUrlQuery>

namespace calyx::network {

AppleMusicClient::AppleMusicClient(QObject* parent)
    : QObject(parent)
{
}

void AppleMusicClient::fetchTestJSON(const QString& developerToken) {
    emit requestStarted();

    QUrl url;
    QNetworkRequest request;

    if (developerToken.isEmpty()) {
        // Query public iTunes Search API
        url = QUrl("https://itunes.apple.com/search");
        QUrlQuery query;
        query.addQueryItem("term", "Calyx Client Hello World");
        query.addQueryItem("limit", "2");
        url.setQuery(query);
    } else {
        // Query Apple Music catalog charts API
        url = QUrl("https://api.music.apple.com/v1/catalog/us/charts");
        QUrlQuery query;
        query.addQueryItem("types", "songs");
        query.addQueryItem("limit", "2");
        url.setQuery(query);

        request.setRawHeader("Authorization", ("Bearer " + developerToken).toUtf8());
    }

    request.setUrl(url);
    request.setHeader(QNetworkRequest::UserAgentHeader, "CalyxMusicClient/0.1.0");

    QNetworkReply* reply = m_networkManager.get(request);

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();

        QByteArray data = reply->readAll();
        bool success = (reply->error() == QNetworkReply::NoError);

        QString formattedJson;
        QJsonParseError parseError;
        QJsonDocument doc = QJsonDocument::fromJson(data, &parseError);

        if (parseError.error == QJsonParseError::NoError && !doc.isNull()) {
            formattedJson = QString::fromUtf8(doc.toJson(QJsonDocument::Indented));
        } else {
            // Format error detail nicely as JSON
            QJsonObject errObj;
            errObj["error"] = reply->errorString();
            errObj["http_status"] = reply->attribute(QNetworkRequest::HttpStatusCodeAttribute).toInt();
            
            if (!data.isEmpty()) {
                QString rawBody = QString::fromUtf8(data);
                // Try parsing raw body to put inside JSON error object
                QJsonDocument subDoc = QJsonDocument::fromJson(data);
                if (!subDoc.isNull()) {
                    errObj["response_body"] = subDoc.object();
                } else {
                    errObj["response_body"] = rawBody;
                }
            }
            formattedJson = QString::fromUtf8(QJsonDocument(errObj).toJson(QJsonDocument::Indented));
        }

        emit requestFinished(formattedJson, success);
    });
}

} // namespace calyx::network
