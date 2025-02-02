use std::os::unix::ffi::OsStrExt;
use std::{collections::{HashMap, HashSet}, ffi::OsStr, fs::{self, OpenOptions}, io::Read, path::{self, Path, PathBuf}};

use crate::parser;

#[derive(Default, Debug, Clone)]
pub struct IncludeWalker {
    search_directory: Vec<PathBuf>,
    pub graph: HashMap<String, Dependency>
}

#[derive(Default, Debug, Clone)]
pub struct Dependency {
    pub file: String,
    full_path: String,
    pub dependencies: HashSet<String>
}

impl PartialEq for Dependency {
    fn eq(&self, other: &Self) -> bool {
        self.full_path == other.full_path
    }
}

impl Eq for Dependency {}

impl std::hash::Hash for Dependency {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.full_path.hash(state)
    }
}

impl IncludeWalker {
    pub fn new() -> Self {
        Self {
            search_directory: Vec::new(),
            graph: HashMap::new()
        }
    }

    pub fn append_dir<P: AsRef<Path>>(&mut self, path: P) {
        self.search_directory.push(path.as_ref().into());
    }

    pub fn search_include<P: AsRef<Path>>(&self, base_file: P, include_entry: &str) -> Option<PathBuf> {
        let mut base_dir = PathBuf::from(base_file.as_ref());
        base_dir.pop();
        let p0 = base_dir.join(include_entry);
        if fs::exists(&p0).unwrap_or(false) {
            return Some(p0);
        }

        for dir in &self.search_directory {
            let possible_path = dir.join(include_entry);
            if fs::exists(&possible_path).unwrap_or(false) {
                return Some(possible_path);
            }
        }

        None
    }
    pub fn walk<P: AsRef<Path>>(&mut self, path: P) {
        self.walk_impl(path, &mut HashSet::new())
    }

    fn walk_impl<P: AsRef<Path>>(&mut self, path: P, visited: &mut HashSet<String>) {
        let absolute = path::absolute(path.as_ref()).expect("Cannot convert path to absolute somehow").to_string_lossy().to_string();
        // No need to search if we already visited.
        if self.graph.contains_key(&absolute) {
            return
        }

        visited.insert(absolute.clone());
        if fs::exists(path.as_ref()).unwrap_or(false) {
            if let Ok(mut f) = OpenOptions::new().read(true).open(path.as_ref()) {
                let mut buf = String::new();
                f.read_to_string(&mut buf).expect("Failed to read to string");

                let includes = parser::parse_includes(&buf);
                let mut dependency_entry = Dependency::default();
                dependency_entry.file = path.as_ref().with_extension("").to_string_lossy().into();
                dependency_entry.full_path = absolute.clone();
                for include_entry in &includes {
                    if let Some(file) = self.search_include(&path, include_entry) {
                        let file_full_path = path::absolute(&file).expect("Failed to convert full path somehow").to_string_lossy().to_string();
                        if path::absolute(&file).unwrap() == path::absolute(&path).unwrap() {
                            continue;
                        }
                        if absolute.ends_with(".c") {
                            let header = path::absolute(path.as_ref().with_extension("h")).expect("Failed to convert full path somehow");
                            if header == path::absolute(&file).unwrap() {
                                continue;
                            }
                        }
                        if !visited.contains(&file_full_path) {
                            self.walk_impl(&file, visited);
                        }
                        dependency_entry.dependencies.insert(file_full_path.clone());

                    }
                }
                // Also check the corresponding source file.
                if path.as_ref().extension().unwrap_or(OsStr::from_bytes(&[])) == "h" {
                    let c_file_full_path = path::absolute(path.as_ref().with_extension("c")).expect("Failed to convert full path somehow").to_string_lossy().to_string();
                    if !visited.contains(&c_file_full_path) {
                        self.walk_impl(path.as_ref().with_extension("c"), visited);
                    }
                    if self.graph.contains_key(&c_file_full_path) {
                        let dep = self.graph.remove(&c_file_full_path).unwrap();
                        for entry in dep.dependencies {
                            dependency_entry.dependencies.insert(entry);
                        }
                    }
                }
                self.graph.insert(absolute, dependency_entry);
            }
        }
    }
}

