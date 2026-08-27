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
                            [--concurrency N]   chunks in flight; default 24.
                              Peak memory is about 15 + 1.1N MB; 10 holds a
                              download to ~25 MB and costs 25-39% of the speed.
      Install or update. Downloads nothing if already current.
  tapline app info APPID [--json]
      Depots, branches and sizes.
  tapline workshop download APPID ITEMID --dir DIR [--json] [--flat]
                            --flat writes into DIR itself, for e.g. garrysmod/addons
                            --extensions gmad,gmad-zip unpacks/converts .gma as it lands
                            --stream [dir|zip|zip-stored] writes as it downloads,
                              never storing the .gma at all
                            --only GLOB  take matching entries; repeatable
                            --pick PATH  take one exact path; repeatable, and
                              missing it is an error
                            --decode gma|zip  how to read the download
                              --only/--pick fetch only the chunks the selected
                              entries live in, so a filter costs less to run
                              than the whole item. They imply --stream's target,
                              which defaults to unpacking into DIR.
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

/// How many threads may be blocked on the filesystem at once.
///
/// Well under tokio's default of 512, and deliberately: the blocking pool here
/// does one thing, which is `fsync` on a finished file. Measured on a 1.5 GB
/// install, four against the default is 22.5–23.2 MB and 18.2–18.6 s against
/// 24.7–25.1 MB and 18.2–21.7 s — cheaper on both axes, because a thread that
/// only waits on a disk does not go faster for having company.
const BLOCKING_THREADS: usize = 4;

fn main() -> ExitCode {
    // First statement, before the runtime or anything worth keeping: this
    // replaces the process. Roughly halves peak memory; see `tapline::tuning`.
    tapline::retune();

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
                .max_blocking_threads(BLOCKING_THREADS)
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
