extern crate static_filez;

fn main() {
    let target = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("static_files");
    static_filez::create(target)
        .add_dir("./src")
        .build().unrwap();
}
