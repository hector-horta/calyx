//! Command line: `calyx [storefront] [--version|-V] [--check] [--no-update-check]`.

#[derive(Debug, PartialEq)]
pub struct Cli {
    /// Two-letter Apple Music storefront ("us" when none is given).
    pub storefront: String,
    /// `--version` / `-V`: print the version and exit.
    pub version: bool,
    /// `--check`: packaging smoke test, start and exit cleanly.
    pub check: bool,
    /// Ask GitHub for newer releases (off with `--no-update-check`).
    pub update_check: bool,
}

impl Cli {
    /// Arguments after the program name; the storefront is the first one
    /// that is not a flag, so flags may go before or after it.
    pub fn parse(args: &[String]) -> Self {
        let has = |flag: &str| args.iter().any(|a| a == flag);
        Self {
            storefront: args
                .iter()
                .find(|a| !a.starts_with('-'))
                .cloned()
                .unwrap_or_else(|| "us".to_string()),
            version: has("--version") || has("-V"),
            check: has("--check"),
            update_check: !has("--no-update-check"),
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
    fn defaults_to_the_us_storefront_with_the_update_check() {
        let cli = parse(&[]);
        assert_eq!(cli.storefront, "us");
        assert!(cli.update_check && !cli.version && !cli.check);
    }

    #[test]
    fn flags_do_not_take_the_storefront_slot() {
        let cli = parse(&["--no-update-check", "mx"]);
        assert_eq!(cli.storefront, "mx");
        assert!(!cli.update_check);
        assert!(parse(&["-V"]).version);
    }
}
