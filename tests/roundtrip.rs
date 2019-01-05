extern crate tempfile;

use std::collections::HashMap;
use std::fs::{create_dir, File};
use std::io::Write;
use std::process::{Child as ChildProcess, Stdio};
use tempfile::tempdir;

#[test]
fn roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;

    let files: HashMap<String, String> = (0..10)
        .map(|i| (format!("file{}", i), format!("test content for file number {}", i)))
        .collect();

    let test_data_dir = dir.path().join("test-data");
    create_dir(&test_data_dir)?;

    for (name, content) in &files {
        let mut file = File::create(test_data_dir.join(name))?;
        file.write_all(content.as_bytes())?;
    }

    let build = escargot::CargoBuild::new()
        .run()?
        .command()
        .current_dir(dir.path())
        .arg("build")
        .arg(dir.path().join("test-data").as_os_str())
        .arg("files")
        .status()?;

    if !build.success() {
        return Err("failed to build archive".into());
    }

    let server = Server::start(dir.path().join("files.archive").as_os_str())?;
    std::thread::sleep(std::time::Duration::from_millis(200));

    for (name, content) in &files {
        let res = reqwest::Client::builder()
            .gzip(true)
            .timeout(std::time::Duration::from_millis(200))
            .build()?
            .get(&format!("http://localhost:{}/{}", server.port()?, name))
            .send()?
            .text()?;
        assert_eq!(&res, content);
    }

    Ok(())
}

struct Server {
    process: ChildProcess,
}

impl Server {
    /// Somewhat random, lol
    const PORT: u16 = 60814;

    fn start(archive_file: &std::ffi::OsStr) -> Result<Server, Box<dyn std::error::Error>> {
        let process = escargot::CargoBuild::new()
            .run()?
            .command()
            .arg("serve")
            .arg(format!("--port={}", Self::PORT))
            .arg(archive_file)
            .stdout(Stdio::null())
            .spawn()?;

        Ok(Server { process })
    }

    fn port(&self) -> Result<u16, Box<dyn std::error::Error>> {
        Ok(Self::PORT)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.process.kill().unwrap();
    }
}
