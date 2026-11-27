//! Command line: `calyx [storefront] [-u|--login <browser>] [--version|-V] [--check]
//! [--no-update-check] [--no-now-playing]`.

#[derive(Debug, PartialEq)]
pub struct Cli {
    /// Two-letter Apple Music storefront, when one was typed.
    pub storefront: Option<String>,
    /// `--version` / `-V`: print the version and exit.
    pub version: bool,
    /// `--check`: packaging smoke test, start and exit cleanly.
    pub check: bool,
    /// Ask GitHub for newer releases (off with `--no-update-check`).
    pub update_check: bool,
    /// Publish what plays to the OS media controls and `now_playing.json`
    /// (off with `--no-now-playing`).
    pub now_playing: bool,
    /// `-u <browser>` / `--login <browser>`: log in with that browser at start.
    pub login: Option<String>,
}

const LOGIN_FLAGS: [&str; 2] = ["-u", "--login"];

impl Cli {
    /// Arguments after the program name; the storefront is the first one
    /// that is not a flag, so flags may go before or after it.
    pub fn parse(args: &[String]) -> Self {
        let has = |flag: &str| args.iter().any(|a| a == flag);
        let login_at = args.iter().position(|a| LOGIN_FLAGS.contains(&a.as_str()));
        let login = login_at
            .and_then(|i| args.get(i + 1))
            .filter(|v| !v.starts_with('-'))
            .cloned();
        // The browser after -u is not the storefront.
        let value_at = login.as_ref().and(login_at).map(|i| i + 1);
        Self {
            storefront: args
                .iter()
                .enumerate()
                .find(|(i, a)| !a.starts_with('-') && Some(*i) != value_at)
                .map(|(_, a)| a.clone()),
            version: has("--version") || has("-V"),
            check: has("--check"),
            update_check: !has("--no-update-check"),
            now_playing: !has("--no-now-playing"),
            login,
        }
    }
}

#[cfg(test)]
mod tests {
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
}
