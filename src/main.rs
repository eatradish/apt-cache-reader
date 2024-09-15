use std::{
    env::args, fs::{self, read_dir, File}, io::{BufReader, BufWriter}, path::{Path, PathBuf}
};

use ahash::RandomState;

type IndexMap<K, V> = indexmap::IndexMap<K, V, RandomState>;

const PACKAGE_FIELD: &str = "Package";
const APT_LISTS_DIR: &str = "/var/lib/apt/lists";
const PACKAGES_FILE_SUFFIX: &str = "_Packages";

fn main() -> anyhow::Result<()> {
    let query = args().skip(1).collect::<Vec<_>>();

    let pkgs = if !Path::new("./cache").exists() {
        let paths = collect_all_packages_paths()?;
        let pkgs = collect_all_packages(&paths);
        let f = BufWriter::new(File::create("./cache").unwrap());
        bincode::serialize_into(f, &pkgs).unwrap();

        pkgs
    } else {
        let f = BufReader::new(File::open("./cache").unwrap());
        let pkgs = bincode::deserialize_from(f).unwrap();
        pkgs
    };

    for q in query {
        let Some(q) = pkgs.get(&q) else {
            continue;
        };

        println!("{:#?}", q);
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

fn collect_all_packages(paths: &[PathBuf]) -> IndexMap<String, Vec<IndexMap<String, String>>> {
    let mut res = IndexMap::with_hasher(RandomState::new());

    for p in paths {
        let f = fs::read_to_string(p).unwrap();
        let packages_file = oma_debcontrol::parse_str(&f).unwrap();

        for p in packages_file {
            let mut map = IndexMap::with_hasher(ahash::RandomState::new());
            let mut name = None;
            for f in p.fields {
                if f.name == PACKAGE_FIELD {
                    name = Some(f.value.to_string());
                }
                map.insert(f.name.to_string(), f.value);
            }

            let name = name.unwrap();
            if !res.contains_key(&name) {
                res.insert(name, vec![map]);
            } else {
                res.get_mut(&name).unwrap().push(map);
            }
        }

    }

    res
}
