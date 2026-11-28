use super::*;
use std::time::Instant;

fn text(line: &Line) -> String {
    line.spans.iter().map(|s| s.content.as_ref()).collect()
}

#[test]
fn shows_the_version_and_each_update_step() {
    let plain = text(&title_with(Language::Es, None, UpdateState::Idle));
    assert_eq!(plain, format!(" Calyx v{VERSION} "));
    let tag = Some("v9.0.0");
    let es = text(&title_with(Language::Es, tag, UpdateState::Idle));
    assert!(es.ends_with("→ v9.0.0  disponible [V] "), "{es}");
    let en = text(&title_with(Language::En, tag, UpdateState::Idle));
    assert!(en.ends_with("→ v9.0.0  available [V] "), "{en}");
    let asked = text(&title_with(
        Language::Es,
        tag,
        UpdateState::Asked(Instant::now()),
    ));
    assert!(
        asked.contains("presiona V otra vez para instalar (4s)"),
        "{asked}"
    );
    let now = Instant::now();
    let busy = text(&title_with(Language::Es, tag, UpdateState::Installing(now)));
    assert!(
        busy.contains("descargando…") && !busy.contains("[V]"),
        "{busy}"
    );
    let done = text(&title_with(Language::Es, tag, UpdateState::Installed));
    assert!(done.contains("instalada: reinicia calyx"), "{done}");
    let failed = text(&title_with(Language::En, tag, UpdateState::Failed(now)));
    assert!(failed.contains("could not install"), "{failed}");
}

#[test]
fn an_old_question_or_failure_goes_back_to_available() {
    let long_ago = Instant::now() - std::time::Duration::from_secs(60);
    for state in [UpdateState::Asked(long_ago), UpdateState::Failed(long_ago)] {
        let t = text(&title_with(Language::Es, Some("v9.0.0"), state));
        assert!(t.ends_with(" disponible [V] "), "{t}");
    }
}
