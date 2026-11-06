use crate::app::{ActiveView, TuiApp};

impl TuiApp {
    /// Opens the home view: recently searched tracks from the local store.
    pub fn open_home(&mut self) {
        self.tracks = self.store.search_tracks("", 50).unwrap_or_default();
        self.selected_index = 0;
        self.list_state.select(Some(0));
        self.active_view = ActiveView::Home;
        self.say(crate::messages::Msg::RecentOpened);
    }
}
