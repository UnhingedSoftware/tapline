//! Parsing both command lines.
//!
//! steamcmd's grammar is a sequence of `+command arg arg` groups, evaluated in
//! order, where later commands depend on earlier ones having run:
//!
//! ```text
//! tapline +login anonymous +force_install_dir /srv/tf2 +app_update 232250 validate +quit
//! ```
//!
//! Every host tool that drives steamcmd emits something of this shape, which is
//! why it is supported verbatim rather than approximated. A tool that had to
//! change its command line would not be a drop-in replacement.
//!
//! The native grammar is the ordinary subcommand kind, for anything new.
//!
//! # Why this is hand-written
//!
//! `clap` parses one grammar well and this is two, one of which is
//! order-dependent and uses `+` as a command sigil. Expressing that as a clap
//! configuration is more code than parsing it directly, and less clear about
//! what steamcmd actually accepts.

use std::path::PathBuf;
use tapline_ids::{AppId, PublishedFileId};

/// One step of a steamcmd-style command line, in the order it was given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// `+login <account|anonymous>`
    Login {
        /// The account name, or `None` for anonymous.
        account: Option<String>,
    },
    /// `+force_install_dir <path>`
    InstallDir(PathBuf),
    /// `+app_update <appid> [validate]`
    AppUpdate {
        /// Which app.
        app: AppId,
        /// Whether `validate` followed.
        validate: bool,
        /// A `-beta <branch>` argument, when given.
        branch: Option<String>,
    },
    /// `+app_info_print <appid>`
    AppInfo(AppId),
    /// `+workshop_download_item <appid> <publishedfileid>`
    WorkshopDownload {
        /// The app the item belongs to.
        app: AppId,
        /// The item.
        item: PublishedFileId,
    },
    /// `+quit`
    Quit,
}

/// What the CLI was asked to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// steamcmd's grammar: a sequence of steps.
    Script(Vec<Step>),
    /// `tapline app plan <appid> --dir <path>`
    Plan {
        /// Which app.
        app: AppId,
        /// Where it would go.
        dir: PathBuf,
        /// The branch.
        branch: String,
        /// Emit newline-delimited JSON instead of text.
        json: bool,
    },
    /// `tapline app download <appid> --dir <path>`
    Download {
        /// Which app.
        app: AppId,
        /// Where to install it.
        dir: PathBuf,
        /// The branch.
        branch: String,
        /// Verify every chunk on disk rather than trusting the install record.
        validate: bool,
        /// How many chunks to fetch at once, or `None` for the default.
        concurrency: Option<usize>,
        /// Emit newline-delimited JSON.
        json: bool,
    },
    /// `tapline app info <appid>`
    Info {
        /// Which app.
        app: AppId,
        /// Emit JSON.
        json: bool,
    },
    /// `tapline workshop download <appid> <itemid> --dir <path>`
    WorkshopDownload {
        /// Write the item's files straight into `--dir`, with no
        /// `steamapps/workshop/content/...` path built underneath it.
        flat: bool,
        /// The app.
        app: AppId,
        /// The item.
        item: PublishedFileId,
        /// Where to put it.
        dir: PathBuf,
        /// Emit JSON.
        json: bool,
    },
    /// `tapline login --qr`
    Login {
        /// Use the QR flow.
        qr: bool,
        /// The account name, for a password login.
        account: Option<String>,
    },
    /// `tapline whoami`
    WhoAmI,
    /// `tapline --help`
    Help,
    /// `tapline --version`
    Version,
}

/// What went wrong reading the command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgError {
    /// What to tell the user.
    pub message: String,
}

impl ArgError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Parses a command line.
pub fn parse(args: &[String]) -> Result<Command, ArgError> {
    let first = args.first().map(String::as_str);

    match first {
        None => Ok(Command::Help),
        Some("--help" | "-h" | "help") => Ok(Command::Help),
        Some("--version" | "-V") => Ok(Command::Version),
        // A leading `+` means steamcmd's grammar, and nothing else does.
        Some(arg) if arg.starts_with('+') => parse_script(args).map(Command::Script),
        Some(_) => parse_native(args),
    }
}

/// Parses steamcmd's `+command` sequence.
fn parse_script(args: &[String]) -> Result<Vec<Step>, ArgError> {
    let mut steps = Vec::new();
    let mut index = 0;

    while index < args.len() {
        let Some(token) = args.get(index) else { break };
        index += 1;

        let Some(command) = token.strip_prefix('+') else {
            return Err(ArgError::new(format!(
                "expected a +command, found {token:?}"
            )));
        };

        // Everything up to the next `+` belongs to this command.
        let mut operands = Vec::new();
        while let Some(next) = args.get(index) {
            if next.starts_with('+') {
                break;
            }
            operands.push(next.clone());
            index += 1;
        }

        steps.push(step_from(command, &operands)?);
    }
    Ok(steps)
}

/// Builds one step from its command and operands.
fn step_from(command: &str, operands: &[String]) -> Result<Step, ArgError> {
    match command {
        "login" => {
            let account = operands.first().map(String::as_str);
            match account {
                // steamcmd spells anonymous logon exactly this way, and every
                // dedicated-server script in existence uses it.
                Some("anonymous") | None => Ok(Step::Login { account: None }),
                Some(name) => Ok(Step::Login {
                    account: Some(name.to_owned()),
                }),
            }
        }
        "force_install_dir" => operands
            .first()
            .map(|dir| Step::InstallDir(PathBuf::from(dir)))
            .ok_or_else(|| ArgError::new("+force_install_dir needs a directory")),
        "app_update" | "app_install" => {
            let app = operands
                .first()
                .ok_or_else(|| ArgError::new("+app_update needs an app id"))?;
            let app = AppId(
                app.parse()
                    .map_err(|_| ArgError::new(format!("{app:?} is not an app id")))?,
            );

            let mut validate = false;
            let mut branch = None;
            let mut rest = operands.iter().skip(1);
            while let Some(operand) = rest.next() {
                match operand.as_str() {
                    "validate" => validate = true,
                    "-beta" => branch = rest.next().cloned(),
                    // steamcmd ignores what it does not recognise here, and a
                    // drop-in replacement that failed instead would break
                    // scripts that pass extra flags.
                    _ => {}
                }
            }

            Ok(Step::AppUpdate {
                app,
                validate,
                branch,
            })
        }
        "app_info_print" | "app_info_update" => {
            let app = operands
                .first()
                .ok_or_else(|| ArgError::new("+app_info_print needs an app id"))?;
            Ok(Step::AppInfo(AppId(app.parse().map_err(|_| {
                ArgError::new(format!("{app:?} is not an app id"))
            })?)))
        }
        "workshop_download_item" => {
            let app = operands
                .first()
                .ok_or_else(|| ArgError::new("+workshop_download_item needs an app id"))?;
            let item = operands
                .get(1)
                .ok_or_else(|| ArgError::new("+workshop_download_item needs an item id"))?;

            Ok(Step::WorkshopDownload {
                app: AppId(
                    app.parse()
                        .map_err(|_| ArgError::new(format!("{app:?} is not an app id")))?,
                ),
                item: PublishedFileId(
                    item.parse()
                        .map_err(|_| ArgError::new(format!("{item:?} is not an item id")))?,
                ),
            })
        }
        "quit" | "exit" => Ok(Step::Quit),
        other => Err(ArgError::new(format!(
            "unsupported steamcmd command +{other}"
        ))),
    }
}

/// Reads `--name value` and `--flag` options out of a list.
struct Options {
    values: Vec<(String, Option<String>)>,
}

impl Options {
    fn parse(args: &[String]) -> Self {
        let mut values = Vec::new();
        let mut index = 0;
        while index < args.len() {
            let Some(token) = args.get(index) else { break };
            index += 1;

            let Some(name) = token.strip_prefix("--") else {
                continue;
            };
            // `--name=value` and `--name value` both work; scripts use both.
            if let Some((name, value)) = name.split_once('=') {
                values.push((name.to_owned(), Some(value.to_owned())));
                continue;
            }
            let next = args.get(index).filter(|next| !next.starts_with("--"));
            if let Some(value) = next {
                values.push((name.to_owned(), Some(value.clone())));
                index += 1;
            } else {
                values.push((name.to_owned(), None));
            }
        }
        Self { values }
    }

    fn value(&self, name: &str) -> Option<&str> {
        self.values
            .iter()
            .find(|(key, _)| key == name)
            .and_then(|(_, value)| value.as_deref())
    }

    fn flag(&self, name: &str) -> bool {
        self.values.iter().any(|(key, _)| key == name)
    }
}

/// Parses the native subcommand grammar.
fn parse_native(args: &[String]) -> Result<Command, ArgError> {
    let options = Options::parse(args);
    let json = options.flag("json");
    let positional: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|arg| !arg.starts_with("--"))
        .collect();

    let dir = || PathBuf::from(options.value("dir").unwrap_or("."));
    // A bad number is refused rather than silently falling back to the default:
    // someone passing --concurrency wants that value, and quietly using another
    // one turns a typo into a mystery about why the download is slow.
    let concurrency = || -> Result<Option<usize>, ArgError> {
        match options.value("concurrency") {
            // Present but valueless: `--concurrency` on its own asked for
            // something and would otherwise get the default silently.
            None if options.flag("concurrency") => Err(ArgError::new(
                "--concurrency needs a number, like --concurrency 32",
            )),
            None => Ok(None),
            Some(raw) => match raw.parse::<usize>() {
                Ok(0) | Err(_) => Err(ArgError::new(format!(
                    "{raw:?} is not a chunk concurrency; give a positive number"
                ))),
                Ok(value) => Ok(Some(value)),
            },
        }
    };
    let branch = || options.value("branch").unwrap_or("public").to_owned();

    let app_id = |value: Option<&&str>| -> Result<AppId, ArgError> {
        let raw = value.ok_or_else(|| ArgError::new("an app id is required"))?;
        Ok(AppId(raw.parse().map_err(|_| {
            ArgError::new(format!("{raw:?} is not an app id"))
        })?))
    };

    match (positional.first().copied(), positional.get(1).copied()) {
        (Some("app"), Some("plan")) => Ok(Command::Plan {
            app: app_id(positional.get(2))?,
            dir: dir(),
            branch: branch(),
            json,
        }),
        (Some("app"), Some("download" | "update" | "install")) => Ok(Command::Download {
            app: app_id(positional.get(2))?,
            dir: dir(),
            branch: branch(),
            validate: options.flag("validate"),
            concurrency: concurrency()?,
            json,
        }),
        (Some("app"), Some("info")) => Ok(Command::Info {
            app: app_id(positional.get(2))?,
            json,
        }),
        (Some("workshop"), Some("download")) => {
            let item = positional
                .get(3)
                .ok_or_else(|| ArgError::new("an item id is required"))?;
            Ok(Command::WorkshopDownload {
                flat: options.flag("flat"),
                app: app_id(positional.get(2))?,
                item: PublishedFileId(
                    item.parse()
                        .map_err(|_| ArgError::new(format!("{item:?} is not an item id")))?,
                ),
                dir: dir(),
                json,
            })
        }
        (Some("login"), _) => Ok(Command::Login {
            qr: options.flag("qr") || positional.get(1).is_none(),
            account: options.value("account").map(str::to_owned),
        }),
        (Some("whoami"), _) => Ok(Command::WhoAmI),
        (Some(other), _) => Err(ArgError::new(format!("unknown command {other:?}"))),
        (None, _) => Ok(Command::Help),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_owned).collect()
    }

    #[test]
    fn the_command_line_every_dedicated_server_script_uses_parses() {
        // This exact shape appears in LinuxGSM, wings eggs and a thousand
        // hand-written scripts. If it did not parse, none of them could switch.
        let command = parse(&args(
            "+login anonymous +force_install_dir /srv/tf2 +app_update 232250 validate +quit",
        ))
        .expect("must parse");

        assert_eq!(
            command,
            Command::Script(vec![
                Step::Login { account: None },
                Step::InstallDir(PathBuf::from("/srv/tf2")),
                Step::AppUpdate {
                    app: AppId(232_250),
                    validate: true,
                    branch: None,
                },
                Step::Quit,
            ])
        );
    }

    #[test]
    fn a_workshop_download_parses() {
        let command = parse(&args(
            "+login anonymous +workshop_download_item 4000 3790437566 +quit",
        ))
        .expect("must parse");

        assert_eq!(
            command,
            Command::Script(vec![
                Step::Login { account: None },
                Step::WorkshopDownload {
                    app: AppId(4000),
                    item: PublishedFileId(3_790_437_566),
                },
                Step::Quit,
            ])
        );
    }

    #[test]
    fn a_beta_branch_is_carried_through() {
        let command =
            parse(&args("+app_update 232250 -beta prerelease validate")).expect("must parse");
        assert_eq!(
            command,
            Command::Script(vec![Step::AppUpdate {
                app: AppId(232_250),
                validate: true,
                branch: Some("prerelease".to_owned()),
            }])
        );
    }

    #[test]
    fn a_named_account_is_kept() {
        let command = parse(&args("+login someone")).expect("must parse");
        assert_eq!(
            command,
            Command::Script(vec![Step::Login {
                account: Some("someone".to_owned())
            }])
        );
    }

    #[test]
    fn unrecognised_operands_are_ignored_the_way_steamcmd_ignores_them() {
        // A replacement that failed on an extra flag would break scripts that
        // pass one, and steamcmd does not fail on them either.
        let command =
            parse(&args("+app_update 232250 validate -someflag extra")).expect("must parse");
        assert_eq!(
            command,
            Command::Script(vec![Step::AppUpdate {
                app: AppId(232_250),
                validate: true,
                branch: None,
            }])
        );
    }

    #[test]
    fn an_unsupported_steamcmd_command_says_which_one() {
        // Rather than ignoring it: a script asking for something we do not do
        // should be told, not quietly given a different result.
        let error =
            parse(&args("+login anonymous +set_steam_guard_code 12345")).expect_err("must refuse");
        assert!(
            error.message.contains("set_steam_guard_code"),
            "{}",
            error.message
        );
    }

    #[test]
    fn the_native_grammar_parses() {
        assert_eq!(
            parse(&args("app plan 232250 --dir /srv/tf2 --json")).expect("must parse"),
            Command::Plan {
                app: AppId(232_250),
                dir: PathBuf::from("/srv/tf2"),
                branch: "public".to_owned(),
                json: true,
            }
        );

        assert_eq!(
            parse(&args(
                "app download 232250 --dir /srv/tf2 --branch prerelease --validate"
            ))
            .expect("must parse"),
            Command::Download {
                app: AppId(232_250),
                dir: PathBuf::from("/srv/tf2"),
                branch: "prerelease".to_owned(),
                validate: true,
                concurrency: None,
                json: false,
            }
        );
    }

    #[test]
    fn both_option_spellings_work() {
        // Scripts use both, and supporting one is a papercut nobody needs.
        let equals = parse(&args("app plan 232250 --dir=/srv/tf2")).expect("must parse");
        let spaced = parse(&args("app plan 232250 --dir /srv/tf2")).expect("must parse");
        assert_eq!(equals, spaced);
    }

    #[test]
    fn a_bare_invocation_asks_for_help_rather_than_failing() {
        assert_eq!(parse(&[]).expect("must parse"), Command::Help);
        assert_eq!(parse(&args("--help")).expect("must parse"), Command::Help);
        assert_eq!(
            parse(&args("--version")).expect("must parse"),
            Command::Version
        );
    }

    #[test]
    fn a_bad_app_id_names_itself() {
        let error = parse(&args("+app_update not-a-number")).expect_err("must refuse");
        assert!(error.message.contains("not-a-number"), "{}", error.message);
    }

    #[test]
    fn an_unknown_native_command_names_itself() {
        let error = parse(&args("frobnicate 1")).expect_err("must refuse");
        assert!(error.message.contains("frobnicate"), "{}", error.message);
    }

    #[test]
    fn the_chunk_concurrency_can_be_overridden() {
        let parsed = parse(&args("app download 232250 --dir /srv/tf2 --concurrency 64"))
            .expect("must parse");
        match parsed {
            Command::Download { concurrency, .. } => assert_eq!(concurrency, Some(64)),
            other => panic!("wrong command: {other:?}"),
        }
    }

    #[test]
    fn an_unusable_concurrency_is_refused_rather_than_ignored() {
        // Falling back to the default here would turn a typo into a mystery
        // about why the download is slower than the flag asked for.
        for bad in ["abc", "0", "-4", ""] {
            let parsed = parse(&args(&format!(
                "app download 232250 --dir /srv/tf2 --concurrency {bad}"
            )));
            assert!(parsed.is_err(), "--concurrency {bad:?} was accepted");
        }
    }
}
