use std::{path::PathBuf, process::ExitCode};

use clap::Parser;

#[path = "../bundle.rs"]
mod bundle;
#[path = "../pack.rs"]
mod pack;

#[derive(Debug, Parser)]
#[command(
    name = "anthrosim-pack",
    version,
    about = "Package a completed AnthroSim run bundle into one deterministic ZIP archive"
)]
struct Cli {
    /// Completed AnthroSim run directory to package.
    run_dir: PathBuf,

    /// Optional archive output path. Defaults to RUN_DIR.zip beside the run directory.
    #[arg(long)]
    output: Option<PathBuf>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match pack::pack_completed_run(&cli.run_dir, cli.output.as_deref()) {
        Ok(path) => {
            println!("wrote run archive {}", path.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("anthrosim-pack: {error}");
            ExitCode::FAILURE
        }
    }
}
