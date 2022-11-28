mod threading;
use std::{fmt::Display, fs};

trait Searchable {
    fn search(&self, query: &str, recursive: &bool) -> Option<Vec<GreprMatch>>;
}

pub struct GreprMatch {
    pathname: String,
    line_num: usize,
    line: String,
}

impl Display for GreprMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{} | {}", self.line_num, self.pathname, self.line)
    }
}

impl GreprMatch {
    fn new(pathname: &str, line_num: usize, line: &str) -> GreprMatch {
        GreprMatch {
            line_num: line_num + 1,
            pathname: String::from(pathname),
            line: String::from(line),
        }
    }
}

pub struct File {
    pathname: String,
}

impl Searchable for File {
    fn search(&self, query: &str, _: &bool) -> Option<Vec<GreprMatch>> {
        let contents = fs::read_to_string(&self.pathname);
        if contents.is_err() {
            return None;
        }
        let contents = contents.unwrap();
        let matches: Vec<GreprMatch> = contents
            .lines()
            .enumerate()
            .into_iter()
            .filter(|(_, line)| line.contains(query))
            .map(|(line_num, line)| GreprMatch::new(&self.pathname, line_num, line))
            .collect();

        Some(matches)
    }
}
impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "File: {}", self.pathname)
    }
}

pub struct Directory {
    pathname: String,
}

impl Display for Directory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Directory: {}", self.pathname)
    }
}

impl Searchable for Directory {
    fn search(&self, query: &str, recursive: &bool) -> Option<Vec<GreprMatch>> {
        let mut matches: Vec<GreprMatch> = Vec::new();

        let paths = fs::read_dir(&self.pathname);
        if paths.is_err() {
            return None;
        }
        let paths = paths.unwrap();
        for path in paths {
            let path = match path {
                Ok(p) => p,
                Err(_) => continue,
            };

            if let Ok(file_type) = path.file_type() {
                if file_type.is_file() {
                    let file = File {
                        pathname: String::from(format!(
                            "{}/{}",
                            self.pathname,
                            path.file_name().to_str().unwrap()
                        )),
                    };

                    if let Some(mut results) = file.search(query, recursive) {
                        (&mut matches).append(&mut results);
                    }
                } else if file_type.is_dir() && *recursive {
                    let dir = Directory {
                        pathname: String::from(format!(
                            "{}/{}",
                            self.pathname,
                            path.file_name().to_str().unwrap()
                        )),
                    };

                    if let Some(mut results) = dir.search(query, recursive) {
                        (&mut matches).append(&mut results);
                    }
                }
            }
        }

        matches.sort_by_key(|m| m.pathname.clone());
        Some(matches)
    }
}

pub struct Config {
    pub query: String,
    pub files: Vec<File>,
    pub directories: Vec<Directory>,
    pub recursive: bool,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();

        let query = match args.next() {
            Some(s) => s,
            None => return Err("Not enough arguments supplied"),
        };

        let mut paths = Vec::new();
        for path in args {
            paths.push(path);
        }

        if paths.is_empty() {
            return Err("Not enough arguments supplied");
        }

        let mut files: Vec<File> = Vec::new();
        let mut directories: Vec<Directory> = Vec::new();
        let mut recursive = false;

        filter_paths(&mut files, &mut directories, &mut recursive, &paths);

        Ok(Config {
            query,
            files,
            directories,
            recursive,
        })
    }
}

fn filter_paths(
    files: &mut Vec<File>,
    directories: &mut Vec<Directory>,
    recursive: &mut bool,
    paths: &[String],
) {
    for path in paths {
        if path.eq("-r") {
            *recursive = true;
        }

        let metadata = match fs::metadata(path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        if metadata.is_dir() {
            directories.push(Directory {
                pathname: String::from(path),
            })
        } else if metadata.is_file() {
            files.push(File {
                pathname: String::from(path),
            })
        } else {
            continue;
        }
    }
}

pub fn run(config: Config) {
    let mut matches: Vec<GreprMatch> = Vec::new();

    for file in &config.files {
        let results = file.search(&config.query, &config.recursive);
        if results.is_some() {
            (&mut matches).append(&mut results.unwrap())
        }
    }

    for dir in &config.directories {
        let results = dir.search(&config.query, &config.recursive);
        if results.is_some() {
            (&mut matches).append(&mut results.unwrap())
        }
    }

    for grepr_match in &matches {
        println!("{}", grepr_match)
    }
}
