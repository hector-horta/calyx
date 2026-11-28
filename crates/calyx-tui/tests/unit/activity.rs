use super::*;

#[test]
fn spinner_runs_until_every_job_finishes() {
    let mut a = Activity::default();
    assert_eq!(a.decorate_jobs(&[Job::Audio], "Listo"), "Listo");
    a.start(Job::Audio);
    a.start(Job::Write);
    a.start(Job::Audio); // replaced song: still one Audio job
    a.finish(Job::Audio);
    assert!(a.spinner().is_some());
    assert!(a.busy(&[Job::Write]) && !a.busy(&[Job::Audio]));
    // The audio zone shows no spinner for a write in progress.
    assert_eq!(a.decorate_jobs(&[Job::Audio], "x"), "x");
    a.finish(Job::Write);
    assert!(a.spinner().is_none());
}

#[test]
fn counter_follows_the_shared_progress() {
    let mut a = Activity::default();
    let p = a.start_counted();
    assert!(a.counter().is_none());
    p.add(300);
    assert_eq!(a.counter().as_deref(), Some("300"));
    p.set_total(1000);
    let text = a.decorate_jobs(&[Job::List], "Cargando Biblioteca...");
    assert!(text.ends_with(" Cargando Biblioteca... 300/1000"), "{text}");
    a.finish(Job::List);
    assert_eq!(
        a.decorate_jobs(&[Job::List], "Biblioteca: 1000 pistas."),
        "Biblioteca: 1000 pistas."
    );
}
