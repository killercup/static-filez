use std::fs::File;
use std::path::Path;
use std::result::Result;

use quicli::prelude::*;

use fst::Map;
use memmap::Mmap;

use slice;

pub struct Site {
    index: Map,
    archive: Mmap,
}

impl Site {
    pub fn new(path: &Path) -> Result<Site, Error> {
        let index_path = path.with_extension("index");
        let archive_path = path.with_extension("archive");

        let index_content = ::std::fs::read(&index_path).with_context(|e| {
            format!("Could not read index file {}: {}", index_path.display(), e)
        })?;
        let index = Map::from_bytes(index_content).with_context(|e| {
            format!("Could not parse index file {}: {}", index_path.display(), e)
        })?;

        let archive_file = File::open(&archive_path).with_context(|e| {
            format!(
                "Could not read archive file {}: {}",
                archive_path.display(),
                e
            )
        })?;
        let archive = unsafe { Mmap::map(&archive_file) }.with_context(|e| {
            format!(
                "Could not open archive file {}: {}",
                archive_path.display(),
                e
            )
        })?;

        Ok(Site { index, archive })
    }

    pub fn get(&self, path: &str) -> Option<&[u8]> {
        let raw_slice = self.index.get(path).or_else(|| {
            let key = format!("{}/index.html", path);
            self.index.get(key)
        })?;
        let (offset, len) = slice::unpack_from_u64(raw_slice);
        let content = self.archive.get(offset..offset + len)?;

        Some(content)
    }
}
