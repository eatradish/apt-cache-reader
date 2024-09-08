use std::{
    env::args, fs::{self, read_dir}, path::PathBuf
};

use ahash::AHashMap;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

fn main() -> anyhow::Result<()> {
    let query = args().skip(1).collect::<Vec<_>>();
    let paths = collect_all_packages_paths()?;

    let pkgs = collect_all_packages(&paths)?;

    for i in pkgs {
        if i.get("Package").is_some_and(|x| query.contains(x)) {
            println!("{:#?}", i);
        }
    }

    Ok(())
}

fn collect_all_packages_paths() -> anyhow::Result<Vec<PathBuf>> {
    let mut paths = vec![];
    for i in read_dir("/var/lib/apt/lists")? {
        let i = i?;
        if i.file_name().to_string_lossy().ends_with("_Packages") {
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
