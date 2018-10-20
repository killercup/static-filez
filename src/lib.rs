extern crate bincode;
extern crate deflate;
extern crate quicli;
extern crate futures;
extern crate hyper;
extern crate mime_guess;
#[macro_use]
extern crate serde_derive;
extern crate tokio;
extern crate walkdir;
extern crate fst;
extern crate clap_port_flag;
extern crate memmap;

use std::fs::File;
use std::path::{Path, PathBuf};
use std::result::Result;

use quicli::prelude::*;
use walkdir::WalkDir;

mod slice;

pub use server::serve;
mod server;

mod site;
pub use site::Site;

pub fn build(src: &Path, target: &Path) -> Result<(), Error> {
    use site::write::{PageMap, Site};
    use std::io::{BufWriter, Write};

    ensure!(src.is_dir(), "Directory `{}` doesn't exist", src.display());

    let index = BufWriter::new(
        File::create(&target.with_extension("index"))
            .with_context(|e| format!("couldn't create file `{}`: {}", target.display(), e))?,
    );
    let mut index = fst::MapBuilder::new(index)
        .with_context(|e| format!("couldn't create index file `{}`: {}", target.display(), e))?;
    let mut archive = BufWriter::new(
        File::create(&target.with_extension("archive"))
            .with_context(|e| format!("couldn't create file `{}`: {}", target.display(), e))?,
    );
    let mut archive_index = 0;

    WalkDir::new(src)
        .sort_by(|a,b| a.strip_prefix(src).cmp(b.strip_prefix(src)))
        .into_iter()
        .flat_map(|entry| entry.map_err(|e| warn!("Couldn't read dir entry {}", e)))
        .filter(|f| f.path().is_file())
        .flat_map(|file| -> Result<PathBuf, ()> {
            let path = file.path();
            Ok(path.strip_prefix(src)
                .map_err(|e| warn!("Couldn't get relative path for `{:?}`: {}", file, e))?
                .to_path_buf())
        })
        .try_fold(index, |mut map, path| {
            let file_content = get_compressed_content(&path)?;
            archive.write_all(&file_content);
            map.insert(path.as_os_str().as_bytes(), slice::pack_in_u64(archive_index, file_content.len()));
            archive_index += file_content.len();
        });

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
