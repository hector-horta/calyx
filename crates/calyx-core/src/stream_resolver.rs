//! One strategy to get playable audio for a track, shared by playback and
//! prefetch: cache → full DRM stream (with retry and a same-song catalog
//! fallback) → 30 s preview.

use std::time::Duration;

use crate::api::AppleMusicClient;
use crate::audio_cache::AudioCache;
use crate::auth::SessionConfig;
use crate::backend::{AudioSource, ResolvedAudio};
use crate::error::{CalyxError, CalyxResult};
use crate::models::Track;

/// Ids worth trying for a track: its own id and, if different, the
/// library/catalog id stored in `hls_manifest_url`.
fn candidate_ids(track: &Track) -> Vec<String> {
    let mut ids = vec![track.id.clone()];
    if let Some(alt) = track
        .hls_manifest_url
        .as_ref()
        .filter(|a| !a.is_empty() && **a != track.id)
    {
        ids.push(alt.clone());
    }
    ids
}

/// Title without "(Remastered)" / "[Live]" style suffixes, for catalog search.
fn search_query(track: &Track) -> String {
    let title = track
        .title
        .split(['(', '['])
        .next()
        .unwrap_or(&track.title)
        .trim();
    format!("{} {}", track.artist, title)
}

fn store(cache: &AudioCache, ids: &[&str], bytes: &[u8]) {
    for id in ids {
        let _ = cache.put(id, bytes);
    }
    cache.evict_if_needed();
}

/// Fills missing preview/artwork URLs from the first same-song catalog hit.
pub(crate) async fn complete_track(client: &AppleMusicClient, mut track: Track) -> Track {
    if track.preview_url.is_some() && track.artwork_url.is_some() {
        return track;
    }
    let Ok(found) = client.search_catalog(&search_query(&track), 3).await else {
        return track;
    };
    if let Some(hit) = found.into_iter().find(|c| track.is_same_song(c)) {
        track.preview_url = track.preview_url.or(hit.preview_url);
        track.artwork_url = track.artwork_url.or(hit.artwork_url);
    }
    track
}

/// Full decrypted stream. `Err(None)`: no credentials; `Err(Some(note))`: why it failed.
async fn fetch_full(
    client: &AppleMusicClient,
    session: &SessionConfig,
    cache: &AudioCache,
    track: &Track,
    ids: &[String],
) -> Result<Vec<u8>, Option<String>> {
    if !session.has_full_stream_credentials() {
        return Err(None);
    }
    let wvd = session
        .load_wvd_device()
        .map_err(|_| Some("Falta device.wvd L3".to_string()))?;
    let mut note = None;
    for id in ids {
        let mut res = client.fetch_full_track_stream(id, &wvd).await;
        if res.is_err() {
            tokio::time::sleep(Duration::from_millis(150)).await;
            res = client.fetch_full_track_stream(id, &wvd).await;
        }
        match res {
            Ok(bytes) => {
                store(cache, &[id.as_str()], &bytes);
                return Ok(bytes);
            }
            Err(e) => note = Some(e.to_string()),
        }
    }
    // Same-song catalog hits only: anything else could be cached under the
    // original id and play the wrong version forever.
    if let Ok(found) = client.search_catalog(&search_query(track), 4).await {
        for cat in found
            .into_iter()
            .filter(|c| track.is_same_song(c) && !ids.contains(&c.id))
        {
            match client.fetch_full_track_stream(&cat.id, &wvd).await {
                Ok(bytes) => {
                    store(cache, &[cat.id.as_str(), track.id.as_str()], &bytes);
                    return Ok(bytes);
                }
                Err(e) => note = Some(e.to_string()),
            }
        }
    }
    Err(note)
}

/// Playable audio for `track`: cache, then full stream, then preview.
pub(crate) async fn resolve_audio(
    client: &AppleMusicClient,
    session: &SessionConfig,
    cache: &AudioCache,
    track: &Track,
) -> CalyxResult<ResolvedAudio> {
    let ids = candidate_ids(track);
    if let Some(bytes) = ids.iter().find_map(|id| cache.get(id)) {
        return Ok(ResolvedAudio {
            bytes,
            source: AudioSource::Cache,
            note: None,
        });
    }
    let note = match fetch_full(client, session, cache, track, &ids).await {
        Ok(bytes) => {
            return Ok(ResolvedAudio {
                bytes,
                source: AudioSource::FullStream,
                note: None,
            });
        }
        Err(note) => note,
    };
    match track.preview_url.as_deref() {
        Some(url) => {
            let bytes = client.fetch_stream_bytes(url).await?;
            Ok(ResolvedAudio {
                bytes,
                source: AudioSource::Preview,
                note,
            })
        }
        None => Err(CalyxError::Audio(
            note.unwrap_or_else(|| "Sin URL de stream".to_string()),
        )),
    }
}

/// Caches the full stream ahead of time; no-op when already cached.
pub(crate) async fn prefetch(
    client: &AppleMusicClient,
    session: &SessionConfig,
    cache: &AudioCache,
    track: &Track,
) {
    let ids = candidate_ids(track);
    if ids.iter().any(|id| cache.contains(id)) {
        return;
    }
    let _ = fetch_full(client, session, cache, track, &ids).await;
}

#[cfg(test)]
#[path = "../tests/unit/stream_resolver.rs"]
mod tests;
