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
use std::path::PathBuf;

pub fn build(src: &Path, target: &Path) -> Result<(), Error> {
    info!(
        "trying to build an index and archive from `{}`",
        src.display()
    );
    let src = src
        .canonicalize()
        .with_context(|_| format!("Cannot canonicalize path `{}`", src.display()))?;

    let src = Box::new(src.to_path_buf());
    let src = &*Box::leak(src);

    ensure!(src.is_dir(), "Directory `{}` doesn't exist", src.display());

    use std::io::{BufWriter, Write};
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
    fn rel_as_bytes(path: &Path) -> Vec<u8> {
        path.to_string_lossy().to_string().into_bytes()
    }
    files.par_sort_by(move |a, b| rel_as_bytes(a.path()).cmp(&rel_as_bytes(b.path())));
    info!("sorted {} files", files.len());

    info!(
        "now building archive {} as well as index {}",
        archive_path.display(),
        index_path.display()
    );

    let files = files
        .chunks(2 << 8)
        .flat_map(|chunk| -> Result<Vec<(PathBuf, Vec<u8>)>, ()> {
            let files: Result<Vec<(PathBuf, Vec<u8>)>, Error> = chunk
                .par_iter()
                .map(|entry| -> Result<(PathBuf, Vec<u8>), Error> {
                    let path = entry.path();
                    let file_content = get_compressed_content(&path).with_context(|_| {
                        format!("Could not read/compress content of {}", path.display())
                    })?;
                    let rel_path = path
                        .strip_prefix(src)
                        .with_context(|_| {
                            format!("Couldn't get relative path for `{:?}`", path.display())
                        })?.to_path_buf();
                    Ok((rel_path, file_content))
                }).collect();
            let mut files = files.map_err(|e| warn!("{}", e))?;

            files.par_sort_by(move |a, b| rel_as_bytes(&a.0).cmp(&rel_as_bytes(&b.0)));
            Ok(files)
        }).flat_map(|xs| xs);

    for (rel_path, file_content) in files {
        archive.write_all(&file_content).with_context(|_| {
            format!(
                "Could not write compressed content to {}",
                archive_path.display()
            )
        })?;

        index
            .insert(
                rel_path.to_string_lossy().as_bytes(),
                slice::pack_in_u64(archive_index, file_content.len()),
            ).with_context(|_| format!("Could not insert file {} into index", rel_path.display()))?;
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
