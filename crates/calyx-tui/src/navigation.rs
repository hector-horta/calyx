use crate::app::{TuiApp, ActiveView};
use calyx_core;

impl TuiApp {
    /// Synchronizes the user's playlists with the server.
    pub async fn sync_playlists(&mut self) {
        let dev_tok = self.session.developer_token.as_deref().unwrap_or("");
        let user_tok = self.session.media_user_token.as_deref().unwrap_or("");
        self.active_view = ActiveView::Playlists;

        match calyx_core::fetch_user_playlists(self.client.http(), dev_tok, user_tok, 50).await {
            Ok(playlists) => {
                self.status_message = format!("Playlists sincronizadas: {} listas encontradas (Enter para abrir).", playlists.len());
                self.playlists = playlists;
                self.playlist_state.select(if self.playlists.is_empty() { None } else { Some(0) });
            }
            Err(e) => {
                self.status_message = format!("Error sincronizando playlists: {}", e);
            }
        }
    }

    /// Synchronizes the user's library of songs with the server.
    pub async fn sync_library(&mut self) {
        let dev_tok = self.session.developer_token.as_deref().unwrap_or("");
        let user_tok = self.session.media_user_token.as_deref().unwrap_or("");
        self.active_view = ActiveView::Library;

        match calyx_core::fetch_user_library_songs(
            self.client.http(),
            dev_tok,
            user_tok,
            &self.session.storefront,
            50
        ).await {
            Ok(songs) => {
                self.status_message = format!("Biblioteca sincronizada: {} canciones cargadas.", songs.len());
                if !songs.is_empty() {
                    self.tracks = songs;
                    self.selected_index = 0;
                    self.list_state.select(Some(0));
                }
            }
            Err(e) => {
                self.status_message = format!("Error sincronizando biblioteca: {}", e);
            }
        }
    }

    /// Opens the home view with default search results.
    pub fn open_home(&mut self) {
        self.tracks = self.store.search_tracks("", 50).unwrap_or_default();
        self.selected_index = 0;
        self.list_state.select(Some(0));
        self.active_view = ActiveView::Home;
        self.status_message = "1 — Recientes: Historial de búsquedas ('/').".to_string();
    }

    /// Opens a section query with the provided search parameters.
    pub async fn open_section_query(&mut self, view: ActiveView, query: &str, label: &str) {
        self.active_view = view;
        if let Ok(res) = self.client.search_catalog(query, 25).await {
            self.tracks = res;
            self.selected_index = 0;
            self.list_state.select(Some(0));
            self.status_message = format!("{label} cargadas ({} pistas).", self.tracks.len());
        }
    }
}