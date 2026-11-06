use super::*;

#[test]
fn messages_follow_the_language() {
    let msg = Msg::PlaylistOpened {
        name: "Mix",
        count: 3,
    };
    assert!(
        msg.text(Language::Es)
            .starts_with("Playlist 'Mix' cargada (3 canciones)")
    );
    assert!(
        msg.text(Language::En)
            .starts_with("Playlist 'Mix' loaded (3 songs)")
    );
}

#[test]
fn album_messages_name_the_keys() {
    let msg = Msg::AlbumOpened {
        name: "Let's Dance",
        count: 8,
        back: "Biblioteca",
    };
    assert!(
        msg.text(Language::Es)
            .contains("'p' lo reproduce, 'A' lo encola, 'o' vuelve a Biblioteca.")
    );
    assert!(
        msg.text(Language::En)
            .contains("'p' plays it, 'A' enqueues it, 'o' goes back to Biblioteca.")
    );
    assert_eq!(
        Msg::BackToList("Biblioteca").text(Language::Es),
        "De vuelta en Biblioteca."
    );
    let added = Msg::AlbumAddedToLibrary("Let's Dance");
    assert!(
        added
            .text(Language::Es)
            .contains("agregado a tu biblioteca ('4'")
    );
    assert!(
        added
            .text(Language::En)
            .contains("added to your library ('4'")
    );
}

#[test]
fn playlist_picker_messages() {
    let pick = Msg::PickPlaylist("Modern Love");
    assert!(
        pick.text(Language::Es)
            .contains("(Enter agrega, Esc cancela)")
    );
    assert!(
        pick.text(Language::En)
            .contains("(Enter adds, Esc cancels)")
    );
    let added = Msg::AddedToPlaylist {
        title: "Modern Love",
        playlist: "Mix",
    };
    assert_eq!(
        added.text(Language::Es),
        "'Modern Love' agregada a la playlist 'Mix'."
    );
}

#[test]
fn list_kind_is_localized_inside_messages() {
    let msg = Msg::ListLoaded {
        kind: ListKind::Library,
        count: 50,
    };
    assert_eq!(msg.text(Language::Es), "Biblioteca: 50 pistas.");
    assert_eq!(msg.text(Language::En), "Library: 50 tracks.");
}
