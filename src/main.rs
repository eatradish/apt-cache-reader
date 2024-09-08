use std::{
    env::args,
    fs::{self, read_dir},
    path::PathBuf,
};

use ahash::AHashMap;
use aho_corasick::AhoCorasick;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

const PACKAGE_FIELD: &str = "Package";
const APT_LISTS_DIR: &str = "/var/lib/apt/lists";
const PACKAGES_FILE_SUFFIX: &str = "_Packages";

fn main() -> anyhow::Result<()> {
    let query = args().skip(1).collect::<Vec<_>>();
    let paths = collect_all_packages_paths()?;
    let ac = AhoCorasick::new(query)?;
    let pkgs = collect_all_packages(&paths)?;

    for i in pkgs {
        if i.get(PACKAGE_FIELD).is_some_and(|x| ac.is_match(x)) {
            println!("{:#?}", i);
        }
    }

    Ok(())
}

fn collect_all_packages_paths() -> anyhow::Result<Vec<PathBuf>> {
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

fn collect_all_packages(paths: &[PathBuf]) -> anyhow::Result<Vec<AHashMap<String, String>>> {
    let pkgs = paths
        .par_iter()
        .filter_map(|p| {
            let mut v = vec![];
            let f = fs::read_to_string(p).ok()?;

            let packages_file = oma_debcontrol::parse_str(&f).ok()?;

            for p in packages_file {
                let mut map = AHashMap::new();
                for f in p.fields {
                    map.insert(f.name.to_string(), f.value);
                }
                v.push(map)
            }

            Some(v)
        })
        .flatten()
        .collect::<Vec<_>>();

    Ok(pkgs)
}
