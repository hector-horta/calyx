use super::Cli;

fn parse(args: &[&str]) -> Cli {
    Cli::parse(&args.iter().map(|a| a.to_string()).collect::<Vec<_>>())
}

#[test]
fn defaults_to_no_storefront_with_the_update_check() {
    let cli = parse(&[]);
    assert_eq!(cli.storefront, None);
    assert!(cli.update_check && cli.now_playing && !cli.version && !cli.check);
    assert!(!parse(&["--no-now-playing"]).now_playing);
}

#[test]
fn flags_do_not_take_the_storefront_slot() {
    let cli = parse(&["--no-update-check", "mx"]);
    assert_eq!(cli.storefront.as_deref(), Some("mx"));
    assert!(!cli.update_check);
    assert!(parse(&["-V"]).version);
}

#[test]
fn login_takes_the_next_argument_as_the_browser() {
    let cli = parse(&["-u", "firefox", "mx"]);
    assert_eq!(cli.login.as_deref(), Some("firefox"));
    assert_eq!(cli.storefront.as_deref(), Some("mx"));
    let cli = parse(&["jp", "--login", "safari"]);
    assert_eq!(
        (cli.login.as_deref(), cli.storefront.as_deref()),
        (Some("safari"), Some("jp"))
    );
    assert_eq!(parse(&["-u"]).login, None);
    assert_eq!(parse(&["mx"]).login, None);
}
