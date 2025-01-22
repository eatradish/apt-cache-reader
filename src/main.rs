use std::{
    env::args,
    fs::{self, read_dir},
    path::PathBuf,
    time::Instant,
};

use ahash::HashMap;
use anyhow::Result;
use deb822_lossless::Deb822;
use debian_control::Binary;

const APT_LISTS_DIR: &str = "/var/lib/apt/lists";
const PACKAGES_FILE_SUFFIX: &str = "_Packages";

fn main() -> Result<()> {
    let query = args().skip(1);

    let paths = collect_all_packages_paths()?;
    let pkgs = collect_all_packages(&paths)?;

    for q in query {
        let now = Instant::now();
        let Some(binarys) = pkgs.get(&q) else {
            continue;
        };

        for b in binarys {
            let p = b.as_deb822();
            println!("{}", p);
        }

        // let Some(desc) = binarys.first().unwrap().description() else {
        //     continue;
        // };

        // println!("{}", desc);
        println!("timer: {}s", now.elapsed().as_secs_f64());
    }

    Ok(())
}

fn collect_all_packages_paths() -> Result<Vec<PathBuf>> {
    let mut paths = vec![];
    for i in read_dir(APT_LISTS_DIR)? {
        let i = i?;
        if i.file_name()
            .to_string_lossy()
            .ends_with(PACKAGES_FILE_SUFFIX)
        {
            paths.push(i.path());
        }
    }

    Ok(paths)
}

fn collect_all_packages(paths: &[PathBuf]) -> Result<HashMap<String, Vec<Binary>>> {
    let mut res: HashMap<_, Vec<_>> = HashMap::with_hasher(ahash::RandomState::new());

    for path in paths {
        let f = fs::read_to_string(path)?;
        let control: Deb822 = f.parse()?;

        for p in control.paragraphs() {
            let binary: Binary = p.into();
            let Some(name) = binary.name() else {
                continue;
            };
            res.entry(name).or_default().push(binary);
        }
    }

    Ok(res)
}
