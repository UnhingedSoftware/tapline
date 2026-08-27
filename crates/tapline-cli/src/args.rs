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
    /// `tapline workshop search <appid> [--text ...] [--tag ...]`
    WorkshopSearch {
        /// Which app's Workshop.
        app: AppId,
        /// Free text to match.
        text: Option<String>,
        /// Tags an item must carry.
        tags: Vec<String>,
        /// Tags that exclude an item.
        exclude_tags: Vec<String>,
        /// Require every tag rather than any.
        all_tags: bool,
        /// How to order results.
        sort: Option<String>,
        /// How many to return.
        limit: Option<u32>,
        /// Where to resume from.
        cursor: Option<String>,
        /// Emit JSON.
        json: bool,
    },
    /// `tapline workshop info <itemid>...`
    WorkshopInfo {
        /// The items to describe.
        items: Vec<PublishedFileId>,
        /// Emit JSON.
        json: bool,
    },
    /// `tapline workshop download <appid> <itemid> --dir <path>`
    WorkshopDownload {
        /// Write the item's files straight into `--dir`, with no
        /// `steamapps/workshop/content/...` path built underneath it.
        flat: bool,
        /// Extensions to run on each file, by name.
        extensions: Vec<String>,
        /// Where to stream the archive, if streaming: "dir", "zip" or
        /// "zip-stored". `None` downloads the archive normally.
        stream: Option<String>,
        /// Globs selecting entries from the archive. Empty takes everything.
        ///
        /// Any selection turns the download into a pipeline, which fetches only
        /// the chunks the selected entries live in.
        only: Vec<String>,
        /// Exact paths to take. Missing one is an error, unlike a glob.
        pick: Vec<String>,
        /// The format to read the download as. `None` means `gma`.
        decode: Option<String>,
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

    /// Every value given for a repeatable option, in the order written.
    ///
    /// `--only a --only b` is two selections, not the second overriding the
    /// first, because a filter list is a union and dropping one silently would
    /// quietly change what gets downloaded.
    fn all_values(&self, name: &str) -> Vec<String> {
        self.values
            .iter()
            .filter(|(key, _)| key == name)
            .filter_map(|(_, value)| value.clone())
            .collect()
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
    // `--stream` alone means a directory; `--stream zip` picks a target. An
    // unrecognised one is refused rather than quietly downloading normally,
    // which would look like the flag did nothing.
    let stream_target = |options: &Options| -> Result<Option<String>, ArgError> {
        if !options.flag("stream") {
            return Ok(None);
        }
        match options.value("stream") {
            None | Some("dir") | Some("directory") => Ok(Some("dir".to_owned())),
            Some("zip") => Ok(Some("zip".to_owned())),
            Some("zip-stored") => Ok(Some("zip-stored".to_owned())),
            Some(other) => Err(ArgError::new(format!(
                "unknown --stream target {other:?}; known: dir, zip, zip-stored"
            ))),
        }
    };
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
        (Some("workshop"), Some("search")) => Ok(Command::WorkshopSearch {
            app: app_id(positional.get(2))?,
            text: options.value("text").map(str::to_owned),
            tags: options.all_values("tag"),
            exclude_tags: options.all_values("exclude-tag"),
            all_tags: options.flag("all-tags"),
            sort: options.value("sort").map(str::to_owned),
            limit: match options.value("limit") {
                None => None,
                Some(raw) => Some(raw.parse().map_err(|_| {
                    ArgError::new(format!(
                        "{raw:?} is not a result count; give a positive number"
                    ))
                })?),
            },
            cursor: options.value("cursor").map(str::to_owned),
            json,
        }),
        (Some("workshop"), Some("info")) => {
            let items: Result<Vec<PublishedFileId>, ArgError> = positional
                .iter()
                .skip(2)
                .map(|raw| {
                    raw.parse()
                        .map(PublishedFileId)
                        .map_err(|_| ArgError::new(format!("{raw:?} is not an item id")))
                })
                .collect();
            let items = items?;
            if items.is_empty() {
                return Err(ArgError::new("at least one item id is required"));
            }
            Ok(Command::WorkshopInfo { items, json })
        }
        (Some("workshop"), Some("download")) => {
            let item = positional
                .get(3)
                .ok_or_else(|| ArgError::new("an item id is required"))?;
            let only = options.all_values("only");
            let pick = options.all_values("pick");
            // Refused rather than ignored: an extension acts on the downloaded
            // archive, and a filtered pipeline never writes one. Silently
            // dropping the flag would look like the extension had run.
            if (!only.is_empty() || !pick.is_empty()) && options.flag("extensions") {
                return Err(ArgError::new(
                    "--extensions acts on a downloaded archive, and --only/--pick \
                     never write one; drop one of them",
                ));
            }
            Ok(Command::WorkshopDownload {
                flat: options.flag("flat"),
                stream: stream_target(&options)?,
                only,
                pick,
                decode: options.value("decode").map(str::to_owned),
                extensions: options
                    .value("extensions")
                    .map(|list| {
                        list.split(',')
                            .map(str::trim)
                            .filter(|name| !name.is_empty())
                            .map(str::to_owned)
                            .collect()
                    })
                    .unwrap_or_default(),
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
    fn a_workshop_search_parses_its_filters() {
        let parsed = parse(&args(
            "workshop search 4000 --text stargate --tag Fun --tag Tool \
             --exclude-tag NSFW --sort trend --limit 5",
        ))
        .expect("parse");
        match parsed {
            Command::WorkshopSearch {
                app,
                text,
                tags,
                exclude_tags,
                sort,
                limit,
                ..
            } => {
                assert_eq!(app.get(), 4000);
                assert_eq!(text.as_deref(), Some("stargate"));
                assert_eq!(tags, vec!["Fun".to_owned(), "Tool".to_owned()]);
                assert_eq!(exclude_tags, vec!["NSFW".to_owned()]);
                assert_eq!(sort.as_deref(), Some("trend"));
                assert_eq!(limit, Some(5));
            }
            other => panic!("wrong command: {other:?}"),
        }
    }

    #[test]
    fn a_search_cursor_survives_its_own_punctuation() {
        // Steam's cursors are base64 and carry `+`, `/` and `=`. A cursor
        // mangled in parsing silently returns the first page again, which
        // reads as "paging is broken" rather than "the flag was eaten".
        let parsed = parse(&args(
            "workshop search 4000 --cursor AoMITpIrrFfJsRd4xNDoAw==",
        ))
        .expect("parse");
        match parsed {
            Command::WorkshopSearch { cursor, .. } => {
                assert_eq!(cursor.as_deref(), Some("AoMITpIrrFfJsRd4xNDoAw=="));
            }
            other => panic!("wrong command: {other:?}"),
        }
    }

    #[test]
    fn workshop_info_takes_several_ids() {
        let parsed = parse(&args("workshop info 104691717 3790437566")).expect("parse");
        match parsed {
            Command::WorkshopInfo { items, .. } => {
                assert_eq!(
                    items,
                    vec![PublishedFileId(104_691_717), PublishedFileId(3_790_437_566)]
                );
            }
            other => panic!("wrong command: {other:?}"),
        }
    }

    #[test]
    fn workshop_info_without_an_id_is_refused() {
        // Rather than describing nothing and exiting zero.
        let error = parse(&args("workshop info")).expect_err("must refuse");
        assert!(error.message.contains("item id"), "{}", error.message);
    }

    #[test]
    fn an_unusable_search_limit_is_refused_rather_than_ignored() {
        for bad in ["zero", "-4", "1.5"] {
            let parsed = parse(&args(&format!("workshop search 4000 --limit {bad}")));
            assert!(parsed.is_err(), "--limit {bad:?} was accepted");
        }
    }

    #[test]
    fn a_repeatable_option_keeps_every_value() {
        // `--only a --only b` is a union. Keeping only the last would silently
        // change what gets downloaded.
        let parsed = parse(&args(
            "workshop download 4000 1 --dir /x --only lua/** --only *.txt",
        ))
        .expect("parse");
        match parsed {
            Command::WorkshopDownload { only, .. } => {
                assert_eq!(only, vec!["lua/**".to_owned(), "*.txt".to_owned()]);
            }
            other => panic!("wrong command: {other:?}"),
        }
    }

    #[test]
    fn picks_are_separate_from_patterns() {
        let parsed = parse(&args(
            "workshop download 4000 1 --dir /x --pick lua/init.lua --only lua/**",
        ))
        .expect("parse");
        match parsed {
            Command::WorkshopDownload { only, pick, .. } => {
                assert_eq!(pick, vec!["lua/init.lua".to_owned()]);
                assert_eq!(only, vec!["lua/**".to_owned()]);
            }
            other => panic!("wrong command: {other:?}"),
        }
    }

    #[test]
    fn a_download_with_no_selection_selects_nothing() {
        // The absence has to be empty rather than "everything", because it is
        // what decides between a plain download and a pipeline.
        let parsed = parse(&args("workshop download 4000 1 --dir /x")).expect("parse");
        match parsed {
            Command::WorkshopDownload {
                only, pick, decode, ..
            } => {
                assert!(only.is_empty() && pick.is_empty());
                assert_eq!(decode, None);
            }
            other => panic!("wrong command: {other:?}"),
        }
    }

    #[test]
    fn a_selection_and_an_extension_together_are_refused() {
        // An extension acts on the downloaded archive and a filtered pipeline
        // never writes one, so accepting both would look like it had run.
        let error = parse(&args(
            "workshop download 4000 1 --dir /x --only lua/** --extensions gmad",
        ))
        .expect_err("must refuse");
        assert!(error.message.contains("--extensions"), "{}", error.message);
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
