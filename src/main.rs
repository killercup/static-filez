extern crate quicli;
extern crate clap_port_flag;
extern crate deflate;
extern crate exitfailure;
extern crate rmp_serde as rmps;
extern crate serde;
extern crate walkdir;
extern crate futures;
extern crate tokio;
extern crate hyper;

use std::path::{Path, PathBuf};
use std::result::Result;
use std::collections::HashMap;
use std::fs::File;

use clap_port_flag::Port;
use exitfailure::ExitFailure;
use quicli::prelude::*;
use walkdir::WalkDir;
use std::sync::Arc;

/// Server static files from a neat small binary
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
            build(&input_dir, &output).context("build failed")?
        }
        Command::Serve { file, port } => serve(&file, &port).context("server failed")?,
    }

    Ok(())
}

fn build(src: &Path, target: &Path) -> Result<(), Error> {
    use rmps::Serializer;
    use serde::Serialize;
    use std::io::BufWriter;

    ensure!(src.is_dir(), "Directory `{}` doesn't exist", src.display());

    let mut file = BufWriter::new(
        File::create(target)
            .with_context(|e| format!("couldn't create file `{}`: {}", target.display(), e))?,
    );
    let mut output = Serializer::new(&mut file);

    let pages: HashMap<String, Vec<u8>> = WalkDir::new(src)
        .into_iter()
        .par_bridge()
        .flat_map(|entry| entry.map_err(|e| warn!("Couldn't read dir entry {}", e)))
        .filter(|f| f.path().is_file())
        .flat_map(|file| -> Result<(String, Vec<u8>), ()> {
            let path = file.path();
            Ok((path.to_string_lossy().into(), get_compressed_content(path).map_err(|e| warn!("{}", e))?))
        })
        .collect();

    ensure!(
        !pages.is_empty(),
        "Would create empty archive. Is the `{}` directory empty?",
        src.display()
    );

    #[derive(Serialize)]
    struct Site {
        pages: HashMap<String, Vec<u8>>,
    }
    let site = Site { pages };
    site.serialize(&mut output)?;

    Ok(())
}

fn get_compressed_content(path: &Path) -> Result<Vec<u8>, Error> {
    use std::fs::read;
    use std::io::Write;

    use deflate::write::GzEncoder;
    use deflate::Compression;

    let data = read(path).with_context(|e| format!("Couldn't read file {}: {}", path.display(), e))?;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::Best);
    encoder.write_all(&data)?;
    let compressed_data = encoder.finish()?;

    Ok(compressed_data)
}

fn serve(path: &Path, port: &Port) -> Result<(), Error> {
    use futures::prelude::*;
    use hyper::{service::service_fn, Body, Response, Server, StatusCode};
    use std::fs::read;
    use rmps::from_slice;

    ensure!(path.is_file(), "File `{}` doesn't exist", path.display());
    let data = read(path).with_context(|e| format!("Couldn't read file {}: {}", path.display(), e))?;
    // That file we just read shall live on util the end of time (or the end of this program, at least)
    let data: &'static [u8] = unsafe {
        let x = std::mem::transmute(data.as_slice());
        std::mem::forget(data);
        x
    };

    #[derive(Deserialize)]
    struct Site<'a> {
        #[serde(borrow)]
        pages: HashMap<&'a str, &'a [u8]>,
    }
    let site: Site = from_slice(data).with_context(|e| format!("Couldn't parse file {}: {}", path.display(), e))?;
    let site = Arc::new((data, site));

    let listener = port.bind()?;

    let handle = tokio::reactor::Handle::current();
    let listener = tokio::net::TcpListener::from_std(listener, &handle)?;
    let addr = listener.local_addr()?;

    let service = move || {
        let site = site.clone();
        service_fn(move |req| {
            let site = &site.1;
            let path = &req.uri().path()[1..];
            let page = site.pages.get(path)
                .or_else(|| {
                    let key = format!("{}/index.html", path);
                    site.pages.get(key.as_str())
                });
            if let Some(&page) = page {
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Transfer-Encoding", "gzip")
                    .body(Body::from(page))
            } else {
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("Not found"))
            }
        })
    };
    let server = Server::builder(listener.incoming())
        .serve(service)
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Server listening on {}", addr);
    tokio::run(server);

    Ok(())
}
