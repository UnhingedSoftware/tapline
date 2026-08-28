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
                            [--concurrency N]   chunks in flight; default 48,
                              the fewest that reach full speed. Peak memory is
                              about 15 + 1.1N MB; 10 holds a download to ~25 MB
                              and costs 30-45% of the speed. Above 48 is slower
                              and dearer.
      Install or update. Downloads nothing if already current.
  tapline app info APPID [--json]
      Depots, branches and sizes.
  tapline workshop search APPID [--text QUERY] [--tag T]... [--exclude-tag T]...
                         [--tag-group T,T]... [--all-tags] [--sort NAME]
                         [--limit N] [--cursor C] [--json]
      Search an app's Workshop. --sort: vote, recent, updated, trend,
      subscribed, text (text needs --text). Paging is a cursor: pass the
      next_cursor from one page as --cursor to get the following one.
      --tag-group is Steam's sidebar: one tag from each group, so
      --tag-group Scene,Video --tag-group Anime means (Scene or Video) and Anime.
  tapline workshop info ITEMID... [--json]
      Describe items by id: title, size, and how they are delivered.
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
      Sign in and save the token, so later commands sign in by themselves.
      --qr        approve in the Steam mobile app; no password is typed
      --account   sign in with a password, typed at the terminal and never
                  taken as an argument
      Only needed for content an account owns; every dedicated server and the
      Workshop search work anonymously. Names the account this machine's Steam
      client last used, if there is one.
  tapline whoami
      Show the current session, any Steam accounts found on this machine, and
      the Steam library paths it is configured with.

Options:
  --json      newline-delimited JSON instead of text
  --help      this
  --version   version
";

/// How many threads may run blocking work at once.
///
/// Two things use this pool, and only one of them waits on a disk: `fsync` on a
/// finished file, and the decrypt-decompress-hash of every chunk. The second is
/// CPU-bound and it is the whole download, so starving this pool throttles the
/// link rather than the disk.
///
/// Measured on a 1.5 GB install at 48 chunks in flight, wire throughput:
///
/// | pool | wall | wire |
/// |---|---|---|
/// | 4 | 11.5 s | 1.02 Gb/s |
/// | 8 | 9.3 s | 1.26 Gb/s |
/// | **16** | **8.4 s** | **1.40 Gb/s** |
/// | 32 | 8.3 s | 1.41 Gb/s |
/// | 64 | 8.5 s | 1.38 Gb/s |
///
/// 16 is where it stops paying: 32 matches it and costs ~9 MB more, and tokio's
/// default of 512 would spawn a thread per blocked task with nothing to show
/// for it.
///
/// This was 4, from a measurement that said 4 was cheaper on both memory and
/// time. That measurement was taken while `--concurrency` was silently capped
/// at 8 by the process-wide budget, where four decode threads genuinely were
/// enough. It is the same trap that produced two wrong concurrency tables: a
/// number tuned under a constraint that no longer applies.
const BLOCKING_THREADS: usize = 16;

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
