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
#[path = "../tests/unit/cli.rs"]
mod tests;
