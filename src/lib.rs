extern crate bincode;
extern crate clap_port_flag;
extern crate deflate;
extern crate fst;
extern crate futures;
extern crate hyper;
extern crate memmap;
extern crate mime_guess;
extern crate quicli;
extern crate tokio;
extern crate walkdir;
#[cfg(test)]
#[macro_use]
extern crate proptest;

use std::fs::File;
use std::path::Path;
use std::result::Result;

use quicli::prelude::*;
use walkdir::WalkDir;

mod slice;

pub use server::serve;
mod server;

mod site;
pub use site::Site;
use walkdir::DirEntry;

pub fn build(src: &Path, target: &Path) -> Result<(), Error> {
    info!(
        "trying to build an index and archive from `{}`",
        src.display()
    );
    use std::io::{BufWriter, Write};
    let src = Box::new(src.to_path_buf());
    let src = &*Box::leak(src);

    ensure!(src.is_dir(), "Directory `{}` doesn't exist", src.display());

    let index_path = target.with_extension("index");
    let index = BufWriter::new(
        File::create(&index_path)
            .with_context(|e| format!("couldn't create file `{}`: {}", target.display(), e))?,
    );
    let mut index = fst::MapBuilder::new(index)
        .with_context(|e| format!("couldn't create index file `{}`: {}", target.display(), e))?;
    info!("will write index to `{}`", index_path.display());

    let archive_path = target.with_extension("archive");
    let mut archive = BufWriter::new(
        File::create(&archive_path)
            .with_context(|e| format!("couldn't create file `{}`: {}", target.display(), e))?,
    );
    info!("will write archive to `{}`", archive_path.display());

    let mut archive_index = 0;

    let mut files = WalkDir::new(src)
        .into_iter()
        .par_bridge()
        .flat_map(|entry| entry.map_err(|e| warn!("Couldn't read dir entry {}", e)))
        .filter(|f| f.path().is_file())
        .collect::<Vec<_>>();

    ensure!(
        !files.is_empty(),
        "Would create empty archive. Is the `{}` directory empty?",
        src.display()
    );
    info!("found {} files", files.len());

    // fst map requires keys to be inserted in lexicographic order _represented as bytes_
    fn rel_as_bytes(p: &DirEntry, src: &Path) -> Vec<u8> {
        p.path()
            .strip_prefix(src)
            .unwrap()
            .to_string_lossy()
            .to_string()
            .into_bytes()
    }
    files.sort_by(move |a, b| rel_as_bytes(a, src).cmp(&rel_as_bytes(b, src)));
    info!("sorted {} files", files.len());

    info!(
        "now building archive {} as well as index {}",
        archive_path.display(),
        index_path.display()
    );
    for file in &files {
        let path = file.path();
        let file_content = get_compressed_content(&path)
            .with_context(|_| format!("Could not read/compress content of {}", path.display()))?;
        archive.write_all(&file_content).with_context(|_| {
            format!(
                "Could not write compressed content to {}",
                archive_path.display()
            )
        })?;

        let rel_path = file
            .path()
            .strip_prefix(src)
            .with_context(|_| format!("Couldn't get relative path for `{:?}`", path.display()))?
            .to_path_buf();

        index
            .insert(
                rel_path.to_string_lossy().as_bytes(),
                slice::pack_in_u64(archive_index, file_content.len()),
            ).with_context(|_| format!("Could not insert file {} into index", path.display()))?;
        archive_index += file_content.len();
    }
    info!("wrote all files");

    index
        .finish()
        .with_context(|e| format!("Could not finish building index: {}", e))?;
    info!("finished index");

    Ok(())
}

fn get_compressed_content(path: &Path) -> Result<Vec<u8>, Error> {
    use std::fs::read;
    use std::io::Write;

    use deflate::write::GzEncoder;
    use deflate::Compression;

    let data =
        read(path).with_context(|e| format!("Couldn't read file {}: {}", path.display(), e))?;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::Best);
    encoder.write_all(&data)?;
    let compressed_data = encoder.finish()?;

    Ok(compressed_data)
}
