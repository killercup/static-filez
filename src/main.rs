extern crate clap_port_flag;
extern crate exitfailure;
extern crate quicli;
extern crate static_filez;

use std::path::PathBuf;
use std::result::Result;

use clap_port_flag::Port;
use exitfailure::ExitFailure;
use quicli::prelude::*;
use std::path::Path;

/// Package static files into a compressed archive and directly serve them over HTTP
#[derive(StructOpt)]
struct Cli {
    #[structopt(flatten)]
    verbosity: Verbosity,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    /// Build a static archive that you can use with `serve` subcommand
    #[structopt(name = "build")]
    Build {
        /// Source directory to archive
        #[structopt(parse(from_os_str))]
        input_dir: PathBuf,
        /// Target file (will write both an `.index` and an `.archive` file)
        #[structopt(parse(from_os_str))]
        output: PathBuf,
    },
    /// Serve a previsouly generated archive over HTTP
    #[structopt(name = "serve")]
    Serve {
        /// Archive to serve (requires both `<name>.index` and `<name>.archive`)
        #[structopt(parse(from_os_str))]
        file: PathBuf,
        #[structopt(flatten)]
        port: Port,
    },
}

fn main() -> Result<(), ExitFailure> {
    let args = Cli::from_args();
    args.verbosity.setup_env_logger(&env!("CARGO_PKG_NAME"))?;

    match args.cmd {
        Command::Build { input_dir, output } => {
            static_filez::build(&input_dir, &output).context("build failed")?
        }
        Command::Serve { file, port } => {
            serve(&file.with_extension(""), &port).context("server failed")?
        }
    }

    Ok(())
}

fn serve(path: &Path, port: &Port) -> Result<(), Error> {
    let site = static_filez::Site::from_path(path)?;

    static_filez::serve(site, port)?;

    Ok(())
}
