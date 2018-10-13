pub mod read {
    use std::collections::HashMap;
    use std::fs::read;
    use quicli::prelude::*;
    use bincode::deserialize;
    use std::path::Path;
    use std::result::Result;

    #[derive(Deserialize)]
    pub struct Site<'a> {
        #[serde(borrow)]
        pub pages: HashMap<&'a str, &'a [u8]>,
    }

    impl Site<'static> {
        pub fn from_file(path: &Path) -> Result<Self, Error> {
            let data = read(path)
                .with_context(|e| format!("Couldn't read file {}: {}", path.display(), e))?
                .into_boxed_slice();
            let data = Box::leak(data);
            let site: Site = deserialize(data)
                .with_context(|e| format!("Couldn't parse file {}: {}", path.display(), e))?;
            Ok(site)
        }
    }
}

pub mod write {
    use std::collections::HashMap;
    use quicli::prelude::*;

    pub type PageMap = HashMap<Box<str>, Box<[u8]>>;

    #[derive(Serialize)]
    pub struct Site {
        pub pages: PageMap,
    }
}