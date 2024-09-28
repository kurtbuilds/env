use std::{fs, io};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Pair {
    pub key: String,
    pub value: String,
}

impl From<&str> for Pair {
    fn from(s: &str) -> Self {
        let mut split = s.splitn(2, '=');
        let pair = (split.next().unwrap().into(), split.next().unwrap().into());
        Pair {
            key: pair.0,
            value: pair.1,
        }
    }
}

impl From<(&str, &str)> for Pair {
    fn from(pair: (&str, &str)) -> Self {
        Pair {
            key: pair.0.into(),
            value: pair.1.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Line {
    Blank,
    Pair(Pair),
    Comment(String),
}

#[derive(Debug)]
pub struct EnvFile {
    pub(crate) lines: Vec<Line>,
    pub path: PathBuf,
    modified: bool,
}


pub fn read(path: impl AsRef<Path>) -> io::Result<EnvFile> {
    EnvFile::read(path)
}

fn parse_lines(s: &str) -> Vec<Line> {
    s.split('\n')
        .map(|line| {
            let line = line.trim();
            if line.starts_with('#') {
                Line::Comment(line.into())
            } else if line.is_empty() {
                Line::Blank
            } else {
                let mut split = line.splitn(2, '=');
                let pair = (split.next().unwrap(), split.next().unwrap()).into();
                Line::Pair(pair)
            }
        })
        .collect()
}

impl EnvFile {
    pub fn parse(s: &str) -> Self {
        EnvFile { lines: parse_lines(s), path: PathBuf::new(), modified: false }
    }

    pub fn read<T: AsRef<Path>>(path: T) -> io::Result<Self> {
        let path = path.as_ref();
        let s = fs::read_to_string(path)?;
        Ok(EnvFile {
            lines: parse_lines(&s),
            path: path.to_path_buf(),
            modified: false,
        })
    }

    pub fn remove(&mut self, key: &str) -> Option<String> {
        let path = self.path.display();
        let mut message = None;
        self.lines.retain(|line| {
            match line {
                Line::Pair(Pair { key: k, .. }) => {
                    if k == key {
                        message = Some(format!("{}: Removed {}", path, key));
                        self.modified = true;
                    }
                    key != key
                }
                _ => true,
            }
        });
        message
    }

    /// Check if a non-empty value exists for the given key
    pub fn has_value(&self, k: &str) -> bool {
        self.lines.iter().any(|p| match p {
            Line::Pair(Pair { key, value }) => k == key && !value.is_empty(),
            _ => false,
        })
    }

    pub fn has_key(&self, k: &str) -> bool {
        self.lines.iter().any(|p| match p {
            Line::Pair(Pair { key, .. }) => k == key,
            _ => false,
        })
    }

    pub fn lookup(&self, lookup: &str) -> Option<&str> {
        self.lines.iter().find_map(|p| match p {
            Line::Pair(Pair { key, value }) => if lookup == key {
                Some(value.as_str())
            } else {
                None
            },
            _ => None,
        })
    }

    /// Returns a message if the key was added or updated
    pub fn add(&mut self, key: &str, value: &str) -> Option<String> {
        for line in &mut self.lines {
            match line {
                Line::Blank => {}
                Line::Pair(Pair { key: k, value: existing_value }) => {
                    if key == k {
                        return if value == existing_value {
                            None
                        } else if value.is_empty() && !existing_value.is_empty() {
                            Some(format!("{}: {} already exists", self.path.display(), key))
                        } else {
                            *line = Line::Pair(Pair { key: key.to_string(), value: value.to_string() });
                            self.modified = true;
                            Some(format!("{}: Updated {}={}", self.path.display(), key, value))
                        };
                    }
                }
                Line::Comment(_) => {}
            }
        }
        self.lines.push(Line::Pair(Pair { key: key.into(), value: value.into() }));
        self.modified = true;
        return Some(format!("{}: Added {}={}", self.path.display(), key, value));
    }

    pub fn save(&self) -> io::Result<()> {
        fs::write(&self.path, self.lines
            .iter()
            .map(|line| match line {
                Line::Blank => String::new(),
                Line::Pair(Pair { key, value }) => format!("{}={}", key, value),
                Line::Comment(line) => line.to_string(),
            })
            .collect::<Vec<String>>()
            .join("\n"),
        )
    }

    pub fn reorder_based_on(&mut self, envfile: &EnvFile) {
        let newlines = envfile.lines.iter()
            .map(|line| match line {
                Line::Blank => Line::Blank,
                Line::Pair(Pair { key, .. }) => {
                    let value = self.lookup(key);
                    if value.is_none() {
                        eprintln!("{}: Added {}=", self.path.display(), key);
                    }
                    Line::Pair(Pair { key: key.to_string(), value: value.unwrap_or_default().to_string() })
                }
                Line::Comment(com) => Line::Comment(com.to_string()),
            })
            .collect();
        self.lines = newlines;
        self.modified = true;
    }

    pub fn iter(&self) -> EnvIter {
        EnvIter {
            env: self,
            i: 0,
        }
    }

    pub fn clone_to_path(&self, path: &Path) -> Self {
        Self {
            lines: self.lines.clone(),
            path: path.to_path_buf(),
            modified: true,
        }
    }

    pub fn save_if_modified(&self) -> io::Result<()> {
        if self.modified {
            self.save()
        } else {
            Ok(())
        }
    }
}

impl<'a> IntoIterator for &'a EnvFile {
    type Item = (&'a str, &'a str);
    type IntoIter = EnvIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        EnvIter {
            env: self,
            i: 0,
        }
    }
}

pub struct EnvIter<'a> {
    env: &'a EnvFile,
    i: usize,
}

impl<'a> Iterator for EnvIter<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        while self.i < self.env.lines.len() {
            let x = unsafe { self.env.lines.get_unchecked(self.i) };
            self.i += 1;
            match x {
                Line::Pair(Pair { key: k, value: v }) => return Some((k.as_str(), v.as_str())),
                _ => {}
            }
        }
        None
    }
}