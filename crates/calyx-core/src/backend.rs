use crate::error::CalyxResult;
use crate::models::Track;
use crate::library::UserPlaylist;
use crate::lyrics::LyricLine;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionType {
    Home,
    NewReleases,
    Radio,
}

#[derive(Debug, Clone)]
pub enum AudioStreamSource {
    EncryptedFmp4 { bytes: Vec<u8> },
    RawEncoded { bytes: Vec<u8>, extension: &'static str },
    PcmDirect { samples: Vec<i16>, sample_rate: u32, channels: u16 },
}

pub trait StreamingBackend: Send + Sync {
    fn service_name(&self) -> &'static str;
    fn search(&self, query: &str, limit: usize) -> impl std::future::Future<Output = CalyxResult<Vec<Track>>> + Send;
    fn fetch_section(&self, section: SectionType) -> impl std::future::Future<Output = CalyxResult<Vec<Track>>> + Send;
    fn fetch_user_library(&self) -> impl std::future::Future<Output = CalyxResult<Vec<Track>>> + Send;
    fn fetch_user_playlists(&self) -> impl std::future::Future<Output = CalyxResult<Vec<UserPlaylist>>> + Send;
    fn fetch_playlist_tracks(&self, playlist_id: &str) -> impl std::future::Future<Output = CalyxResult<Vec<Track>>> + Send;
    fn resolve_stream(&self, track_id: &str) -> impl std::future::Future<Output = CalyxResult<AudioStreamSource>> + Send;
    fn fetch_lyrics(&self, track: &Track) -> impl std::future::Future<Output = CalyxResult<Vec<LyricLine>>> + Send;
    fn login(&mut self) -> impl std::future::Future<Output = CalyxResult<()>> + Send;
    fn is_logged_in(&self) -> bool;
}

pub struct AppleMusicBackend {
    pub client: crate::api::AppleMusicClient,
    pub session: crate::auth::SessionConfig,
}

impl AppleMusicBackend {
    pub fn new(session: crate::auth::SessionConfig) -> CalyxResult<Self> {
        let mut client = crate::api::AppleMusicClient::new(&session.storefront)?;
        client.set_tokens(session.developer_token.clone(), session.media_user_token.clone());
        Ok(Self { client, session })
    }
}

impl StreamingBackend for AppleMusicBackend {
    fn service_name(&self) -> &'static str {
        "Apple Music"
    }

    async fn search(&self, query: &str, limit: usize) -> CalyxResult<Vec<Track>> {
        self.client.search_catalog(query, limit).await
    }

    async fn fetch_section(&self, section: SectionType) -> CalyxResult<Vec<Track>> {
        match section {
            SectionType::Home => self.client.search_catalog("top hits", 50).await,
            SectionType::NewReleases => self.client.search_catalog("top hits 2026", 50).await,
            SectionType::Radio => self.client.search_catalog("electronic chill radio mix", 50).await,
        }
    }

    async fn fetch_user_library(&self) -> CalyxResult<Vec<Track>> {
        let dev = self.session.developer_token.as_deref().unwrap_or_default();
        let usr = self.session.media_user_token.as_deref().unwrap_or_default();
        crate::library::fetch_user_library_songs(self.client.http(), dev, usr, &self.session.storefront, 50).await
    }

    async fn fetch_user_playlists(&self) -> CalyxResult<Vec<UserPlaylist>> {
        let dev = self.session.developer_token.as_deref().unwrap_or_default();
        let usr = self.session.media_user_token.as_deref().unwrap_or_default();
        crate::library::fetch_user_playlists(self.client.http(), dev, usr, 50).await
    }

    async fn fetch_playlist_tracks(&self, playlist_id: &str) -> CalyxResult<Vec<Track>> {
        let dev = self.session.developer_token.as_deref().unwrap_or_default();
        let usr = self.session.media_user_token.as_deref().unwrap_or_default();
        crate::library::fetch_playlist_tracks(self.client.http(), dev, usr, playlist_id, &self.session.storefront).await
    }

    async fn resolve_stream(&self, track_id: &str) -> CalyxResult<AudioStreamSource> {
        let wvd = match self.session.resolve_wvd_path() {
            Some(p) => {
                let b = std::fs::read(&p).map_err(|e| crate::error::CalyxError::Auth(e.to_string()))?;
                crate::drm::WidevineDeviceL3::from_wvd_bytes(&b)?
            }
            None => crate::drm::WidevineDeviceL3::builtin()?,
        };
        let bytes = self.client.fetch_full_track_stream(track_id, &wvd).await?;
        Ok(AudioStreamSource::EncryptedFmp4 { bytes })
    }

    async fn fetch_lyrics(&self, track: &Track) -> CalyxResult<Vec<LyricLine>> {
        let dev = self.session.developer_token.as_deref();
        let usr = self.session.media_user_token.as_deref();
        let apple_ctx = match (dev, usr) {
            (Some(d), Some(u)) => Some((d, u, self.session.storefront.as_str(), track.id.as_str())),
            _ => None,
        };
        let lyrics = crate::lyrics::fetch_track_lyrics(
            self.client.http(), &track.title, &track.artist, apple_ctx
        ).await;
        Ok(lyrics)
    }

    async fn login(&mut self) -> CalyxResult<()> {
        let (dev, user_token, sf, _guard) = self.client.login_and_verify_user().await?;
        self.session.developer_token = Some(dev);
        self.session.media_user_token = Some(user_token);
        self.session.storefront = sf;
        self.session.save()?;
        Ok(())
    }

    fn is_logged_in(&self) -> bool {
        self.session.is_logged_in()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apple_music_backend_metadata() {
        let session = crate::auth::SessionConfig {
            storefront: "us".to_string(),
            ..Default::default()
        };
        let backend = AppleMusicBackend::new(session).unwrap();
        assert_eq!(backend.service_name(), "Apple Music");
        assert!(!backend.is_logged_in());
    }
}
