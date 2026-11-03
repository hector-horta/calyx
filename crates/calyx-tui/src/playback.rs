use calyx_core::Track;
use crate::app::TuiApp;
use crate::artwork::TerminalCoverArt;

pub enum BgEvent {
    AudioReady(Track, Result<Vec<u8>, String>, Vec<calyx_core::LyricLine>, String),
    ArtworkReady(String, TerminalCoverArt),
    PlaylistReady(String, Result<Vec<Track>, String>),
}

impl TuiApp {
    pub async fn poll_bg_events(&mut self) {
        while let Ok(ev) = self.bg_rx.try_recv() {
            match ev {
                BgEvent::AudioReady(track, res, lyrics, hint) => {
                    if self.current_track.as_ref().map(|t| t.id == track.id).unwrap_or(true) {
                        self.current_lyrics = lyrics;
                        match res {
                            Ok(bytes) => match self.audio.play_encoded_bytes(bytes) {
                                Ok(()) => { self.audio.set_volume(self.volume); self.status_message = hint; self.current_track = Some(track); }
                                Err(e) => self.status_message = format!("Error de audio: {e}"),
                            },
                            Err(e) => self.status_message = e,
                        }
                    }
                }
                BgEvent::ArtworkReady(tid, art) => {
                    if self.current_track.as_ref().map(|t| t.id == tid).unwrap_or(true) { self.current_artwork = Some(art); }
                }
                BgEvent::PlaylistReady(name, Ok(list)) if !list.is_empty() => {
                    self.queue.clear();
                    for t in &list { self.queue.enqueue_back(t.clone()); }
                    self.tracks = list; self.selected_index = 0; self.list_state.select(Some(0));
                    self.active_view = crate::app::ActiveView::Library;
                    self.status_message = format!("Reproduciendo '{}' ({} pistas)...", name, self.queue.len());
                    self.play_next().await;
                }
                BgEvent::PlaylistReady(name, _) => self.status_message = format!("No se pudo reproducir playlist '{name}'."),
            }
        }
    }

pub async fn play_track(&mut self, mut track: Track) {
    if let Some(old) = self.play_task.take() { old.abort(); }
    self.audio.stop();
    let orig_id = track.id.clone();
    let seed = format!("{}:{}:{}:{}", orig_id, track.artist, track.album, track.title);
    self.current_artwork = Some(TerminalCoverArt::from_jpeg_or_seed(None, &seed));
    self.current_lyrics = vec![calyx_core::LyricLine { time_ms: 0, text: "⏳ Sincronizando letra...".to_string() }];
    self.current_track = Some(track.clone());
    self.status_message = format!("⏳ Cargando audio en 2do plano: {} — {}...", track.artist, track.title);

    let (client, session, tx, cache) = (self.client.clone(), self.session.clone(), self.bg_tx.clone(), self.audio_cache.clone());
    self.play_task = Some(tokio::spawn(async move {
        let mut candidates = vec![orig_id.clone()];
        if let Some(lib_id) = track.hls_manifest_url.clone().filter(|id| !candidates.contains(id)) {
            candidates.push(lib_id);
        }
        if orig_id.starts_with("i.") || track.preview_url.is_none() || track.artwork_url.is_none() {
            let clean_title = track.title.split(['(', '[']).next().unwrap_or(&track.title).trim();
            if let Ok(found) = client.search_catalog(&format!("{} {}", track.artist, clean_title), 2).await {
                for cat in found {
                    if track.preview_url.is_none() { track.preview_url = cat.preview_url; }
                    if track.artwork_url.is_none() { track.artwork_url = cat.artwork_url; }
                    if candidates.len() < 2 && !candidates.contains(&cat.id) { candidates.push(cat.id); }
                }
            }
        }
        let mut sent = false;
        let mut drm_note: Option<String> = None;

        if let Some(art_url) = track.artwork_url.clone() {
            let (c2, tx2, tid2, s2) = (client.clone(), tx.clone(), track.id.clone(), seed.clone());
            tokio::spawn(async move {
                if let Ok(ab) = c2.fetch_stream_bytes(&art_url).await {
                    let _ = tx2.send(BgEvent::ArtworkReady(tid2, TerminalCoverArt::from_jpeg_or_seed(Some(&ab), &s2)));
                }
            });
        }

        let apple_ctx = match (&session.developer_token, &session.media_user_token) {
            (Some(dev), Some(usr)) => Some((dev.as_str(), usr.as_str(), session.storefront.as_str(), track.id.as_str())),
            (Some(dev), None) => Some((dev.as_str(), "", session.storefront.as_str(), track.id.as_str())),
            _ => None,
        };
        let lyrics_fut = calyx_core::fetch_track_lyrics(client.http(), &track.title, &track.artist, apple_ctx);

        for cid in &candidates {
            if let Some(cached_bytes) = cache.get(cid) {
                let lyrics = lyrics_fut.await;
                let _ = tx.send(BgEvent::AudioReady(track.clone(), Ok(cached_bytes), lyrics, "[AAC 256k | Caché]".to_string()));
                return;
            }
        }

        let wvd_fut = async {
            let mut local_note: Option<String> = None;
            if session.has_full_stream_credentials() {
                match session.load_wvd_device() {
                    Ok(wvd) => {
                        let hint_ok = "[AAC 256k]".to_string();
                        for cid in &candidates {
                            match client.fetch_full_track_stream(cid, &wvd).await {
                                Ok(full_bytes) => {
                                    let (c_c, id_c, b_c) = (cache.clone(), cid.clone(), full_bytes.clone());
                                    tokio::task::spawn_blocking(move || { if c_c.put(&id_c, &b_c).is_ok() { c_c.evict_if_needed(); } });
                                    return Ok((full_bytes, hint_ok));
                                }
                                Err(e) => local_note = Some(e.to_string()),
                            }
                        }
                        let clean_title = track.title.split(['(', '[']).next().unwrap_or(&track.title).trim();
                        let q = format!("{} {}", track.artist, clean_title);
                        if let Ok(found) = client.search_catalog(&q, 4).await {
                            for cat in found {
                                if !candidates.contains(&cat.id) {
                                    match client.fetch_full_track_stream(&cat.id, &wvd).await {
                                        Ok(full_bytes) => {
                                            let (c_c, id_c, b_c) = (cache.clone(), cat.id.clone(), full_bytes.clone());
                                            tokio::task::spawn_blocking(move || { if c_c.put(&id_c, &b_c).is_ok() { c_c.evict_if_needed(); } });
                                            return Ok((full_bytes, hint_ok));
                                        }
                                        Err(e) => local_note = Some(e.to_string()),
                                    }
                                }
                            }
                        } else {
                            local_note = Some("No results found in catalog search".to_string());
                        }
                    }
                    Err(_) => local_note = Some("Falta device.wvd L3 en carpeta".to_string()),
                }
            }
            Err(local_note)
        };

        let (wvd_res, lyrics) = tokio::join!(wvd_fut, lyrics_fut);
        match wvd_res {
            Ok((full_bytes, hint_ok)) => {
                sent = true;
                let _ = tx.send(BgEvent::AudioReady(track.clone(), Ok(full_bytes), lyrics.clone(), hint_ok));
            }
            Err(note) => drm_note = note,
        }

if !sent {
    match track.preview_url.as_deref() {
        Some(u) => {
            let res = client.fetch_stream_bytes(u).await.map_err(|e| format!("Error de red: {e}"));
            let hint = drm_note.map(|n| format!("[Preview 30s | {n}]")).unwrap_or_else(|| "[Preview 30s — Presiona 'u' para Login Apple]".to_string());
            let _ = tx.send(BgEvent::AudioReady(track.clone(), res, lyrics, hint));
        }
        None => {
            let _ = tx.send(BgEvent::AudioReady(
                track.clone(),
                Err("Sin URL de stream.".to_string()),
                lyrics,
                String::new()
            ));
        }
    }
}
    }));
}

pub async fn play_selected(&mut self) {
    let (dev, usr) = (
        self.session.developer_token.as_deref().unwrap_or(""),
        self.session.media_user_token.as_deref().unwrap_or("")
    );

    if self.active_view == crate::app::ActiveView::Playlists {
        if let Some(pl) = self.playlists.get(self.playlist_state.selected().unwrap_or(0)).cloned() {
            match calyx_core::fetch_playlist_tracks(
                self.client.http(),
                dev,
                usr,
                &pl.id,
                &self.session.storefront
            ).await {
                Ok(t) => {
                    self.tracks = t;
                    self.selected_index = 0;
                    self.list_state.select(Some(0));
                    self.active_view = crate::app::ActiveView::Library;
                    self.status_message = format!("Playlist '{}' cargada ({} canciones). Presiona 'p' para reproducir toda.", pl.name, self.tracks.len());
                }
                Err(e) => self.status_message = format!("Error cargando playlist '{}': {e}", pl.name),
            }
        }
    } else if let Some(track) = self.selected_track() {
        self.play_track(track).await;
    }
}

pub async fn play_next(&mut self) {
    if let Some(next) = self.queue.next_track(self.current_track.clone()) {
        self.play_track(next).await;
    }
}

pub async fn play_previous(&mut self) {
    if let Some(prev) = self.queue.previous_track(self.current_track.clone()) {
        self.play_track(prev).await;
    }
}

pub async fn check_auto_advance(&mut self) {
    if self.audio.is_finished() { self.play_next().await; }
}

    pub async fn play_all(&mut self) {
        if self.active_view == crate::app::ActiveView::Playlists {
            if let Some(pl) = self.playlists.get(self.playlist_state.selected().unwrap_or(0)).cloned() {
                self.status_message = format!("⏳ Cargando playlist '{}' en 2do plano...", pl.name);
                let (http, sf, tx) = (self.client.http().clone(), self.session.storefront.clone(), self.bg_tx.clone());
                let dev_tok = self.session.developer_token.clone().unwrap_or_default();
                let user_tok = self.session.media_user_token.clone().unwrap_or_default();
                tokio::spawn(async move {
                    let res = calyx_core::fetch_playlist_tracks(&http, &dev_tok, &user_tok, &pl.id, &sf).await.map_err(|e| e.to_string());
                    let _ = tx.send(BgEvent::PlaylistReady(pl.name, res));
                });
            }
            return;
        }
        if !self.tracks.is_empty() {
            self.queue.clear();
            let start = self.selected_index.min(self.tracks.len() - 1);
            for t in self.tracks[start..].iter().chain(self.tracks[..start].iter()) {
                self.queue.enqueue_back(t.clone());
            }
            self.play_next().await;
        }
    }

pub async fn sync_playlists(&mut self) -> () {
    let dev_tok = self.session.developer_token.as_deref().unwrap_or("");
    let user_tok = self.session.media_user_token.as_deref().unwrap_or("");
    self.active_view = crate::app::ActiveView::Playlists;
    match calyx_core::fetch_user_playlists(self.client.http(), dev_tok, user_tok, 50).await {
        Ok(playlists) => {
            self.status_message = format!("Playlists sincronizadas: {} listas encontradas (Enter para abrir).", playlists.len());
            self.playlists = playlists;
            self.playlist_state.select(if self.playlists.is_empty() { None } else { Some(0) });
        }
        Err(e) => self.status_message = format!("Error sincronizando playlists: {e}"),
    }
}

    pub async fn sync_library(&mut self) {
        let dev_tok = self.session.developer_token.as_deref().unwrap_or("");
        let user_tok = self.session.media_user_token.as_deref().unwrap_or("");
        self.active_view = crate::app::ActiveView::Library;

        match calyx_core::fetch_user_library_songs(self.client.http(), dev_tok, user_tok, &self.session.storefront, 50).await {
            Ok(songs) => {
                self.status_message = format!("Biblioteca sincronizada: {} canciones cargadas.", songs.len());
                if !songs.is_empty() {
                    self.tracks = songs;
                    self.selected_index = 0;
                    self.list_state.select(Some(0));
                }
            }
            Err(e) => self.status_message = format!("Error sincronizando biblioteca: {e}"),
        }
    }

    pub fn open_home(&mut self) {
        self.tracks = self.store.search_tracks("", 50).unwrap_or_default();
        self.selected_index = 0;
        self.list_state.select(Some(0));
        self.active_view = crate::app::ActiveView::Home;
        self.status_message = "1 — Recientes: Historial de búsquedas ('/').".to_string();
    }

    pub async fn open_section_query(&mut self, view: crate::app::ActiveView, query: &str, label: &str) {
        self.active_view = view;
        if let Ok(res) = self.client.search_catalog(query, 25).await {
            self.tracks = res;
            self.selected_index = 0;
            self.list_state.select(Some(0));
            self.status_message = format!("{label} cargadas ({} pistas).", self.tracks.len());
        }
    }
}
