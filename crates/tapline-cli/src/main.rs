//! The `tapline` binary.
//!
//! Speaks steamcmd's command line so existing scripts drop in unchanged, and a
//! native one for anything new:
//!
//! ```sh
//! tapline +login anonymous +force_install_dir /srv/tf2 +app_update 232250 validate +quit
//! tapline app plan 232250 --dir /srv/tf2 --json
//! ```

mod args;
mod run;

use std::process::ExitCode;

/// What to print when asked.
const HELP: &str = "\
tapline — install Steam apps and Workshop content

steamcmd's command line, unchanged:
  tapline +login anonymous +force_install_dir DIR +app_update APPID [validate] +quit
  tapline +login anonymous +workshop_download_item APPID ITEMID +quit

Native:
  tapline app plan APPID --dir DIR [--branch NAME] [--json]
      What an install would cost. Fetches no content.
  tapline app download APPID --dir DIR [--branch NAME] [--validate] [--json]
      Install or update. Downloads nothing if already current.
  tapline app info APPID [--json]
      Depots, branches and sizes.
  tapline workshop download APPID ITEMID --dir DIR [--json]
      One Workshop item.
  tapline login [--qr | --account NAME]
      Sign in. Only needed for apps an account owns; dedicated servers do not.
  tapline whoami
      Show the current session.

Options:
  --json      newline-delimited JSON instead of text
  --help      this
  --version   version
";

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();

    let command = match args::parse(&arguments) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("tapline: {}", error.message);
            eprintln!("try `tapline --help`");
            return ExitCode::FAILURE;
        }
    };

    match command {
        args::Command::Help => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        args::Command::Version => {
            println!("tapline {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        other => {
            // The runtime is created here rather than by a macro on main, so
            // --help and --version cost nothing.
            let runtime = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    eprintln!("tapline: could not start: {error}");
                    return ExitCode::FAILURE;
                }
            };

            match runtime.block_on(run::execute(other)) {
                Ok(()) => ExitCode::SUCCESS,
                Err(message) => {
                    eprintln!("tapline: {message}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}
