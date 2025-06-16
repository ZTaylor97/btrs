use anyhow::{Result, anyhow};

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub kind: FileKind,
}

#[derive(Debug, Clone)]
pub enum FileKind {
    File,
    Directory { children: Vec<FileEntry> },
}

impl FileEntry {
    pub fn new(base_path: &str) -> Self {
        FileEntry {
            name: base_path.into(),
            kind: FileKind::Directory { children: vec![] },
        }
    }

    /// Takes an array of paths for a file and creates the appropriate file hierarchy
    /// in the provided [`FileEntry`] object.
    ///
    /// Returns an [`Error`](`anyhow::Error`) if an insert is attempted on a
    /// leaf node file rather than a directory node.
    pub fn insert_path(&mut self, path: &[String]) -> Result<(), anyhow::Error> {
        let mut current = self;
        for (i, segment) in path.iter().enumerate() {
            match &mut current.kind {
                FileKind::File => {
                    return Err(anyhow!(
                        "Did not expect a FileKind::File when inserting path {segment} for {:?}",
                        current.name
                    ));
                }
                FileKind::Directory { children } => {
                    // Check if path already exists
                    if let Some(pos) = children.iter().position(|e| e.name == *segment) {
                        current = &mut children[pos];
                    } else {
                        // Check if file or directory
                        let is_file = i == path.len() - 1;

                        let new_entry = FileEntry {
                            name: segment.clone(),
                            kind: if is_file {
                                FileKind::File
                            } else {
                                FileKind::Directory { children: vec![] }
                            },
                        };

                        children.push(new_entry);
                        current = children.last_mut().unwrap();
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_path_builds_tree() -> Result<()> {
        let mut root = FileEntry {
            name: "root".to_string(),
            kind: FileKind::Directory { children: vec![] },
        };

        let paths = vec![
            vec!["folder".to_string(), "file1.txt".to_string()],
            vec![
                "folder".to_string(),
                "nested".to_string(),
                "file2.txt".to_string(),
            ],
            vec!["another".to_string(), "file3.txt".to_string()],
        ];

        for path in paths {
            root.insert_path(&path)?;
        }

        // Root should have two children: "folder" and "another"
        let children = match &root.kind {
            FileKind::Directory { children } => children,
            _ => panic!("Root is not a directory"),
        };

        assert_eq!(children.len(), 2);

        // "folder" directory should exist with two children
        let folder = children
            .iter()
            .find(|c| c.name == "folder")
            .expect("folder missing");
        match &folder.kind {
            FileKind::Directory { children } => {
                // folder has two children: "file1.txt" and "nested"
                assert_eq!(children.len(), 2);

                // Check "file1.txt" exists and is a file
                let file1 = children
                    .iter()
                    .find(|c| c.name == "file1.txt")
                    .expect("file1.txt missing");
                assert!(matches!(file1.kind, FileKind::File));

                // Check "nested" directory exists with one child
                let nested = children
                    .iter()
                    .find(|c| c.name == "nested")
                    .expect("nested missing");
                match &nested.kind {
                    FileKind::Directory { children } => {
                        assert_eq!(children.len(), 1);

                        // Check "file2.txt" inside nested is a file
                        let file2 = children
                            .iter()
                            .find(|c| c.name == "file2.txt")
                            .expect("file2.txt missing");
                        assert!(matches!(file2.kind, FileKind::File));
                    }
                    _ => panic!("nested is not a directory"),
                }
            }
            _ => panic!("folder is not a directory"),
        }

        // "another" directory should exist with one child: "file3.txt"
        let another = children
            .iter()
            .find(|c| c.name == "another")
            .expect("another missing");
        match &another.kind {
            FileKind::Directory { children } => {
                assert_eq!(children.len(), 1);
                let file3 = children
                    .iter()
                    .find(|c| c.name == "file3.txt")
                    .expect("file3.txt missing");
                assert!(matches!(file3.kind, FileKind::File));
            }
            _ => panic!("another is not a directory"),
        }

        Ok(())
    }
}
