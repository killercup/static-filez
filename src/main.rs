extern crate static_filez;
extern crate quicli;
extern crate clap_port_flag;
extern crate exitfailure;

use std::path::PathBuf;
use std::result::Result;

use quicli::prelude::*;
use clap_port_flag::Port;
use exitfailure::ExitFailure;
use std::path::Path;

/// Serve static files from a neat small binary
#[derive(StructOpt)]
struct Cli {
    #[structopt(flatten)]
    verbosity: Verbosity,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    /// Build a static file that you can use to serve all your static files
    #[structopt(name = "build")]
    Build {
        /// Source directory
        #[structopt(parse(from_os_str))]
        input_dir: PathBuf,
        /// Target file
        #[structopt(parse(from_os_str))]
        output: PathBuf,
    },
    /// Server them files over HTTP
    #[structopt(name = "serve")]
    Serve {
        /// Archive to serve
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
        Command::Serve { file, port } => serve(&file.with_extension(""), &port).context("server failed")?,
    }

    Ok(())
}

fn serve(path: &Path, port: &Port) -> Result<(), Error> {
    let site = static_filez::Site::new(path)?;

    static_filez::serve(site, port)?;

    Ok(())
}
