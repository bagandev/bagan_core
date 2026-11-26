use super::def_files::DefaultFiles;
use crate::ignore::IgnoreMatcher;
use napi_derive::napi;
use std::{fs, path::PathBuf};

/// Recursively collects files with one of the requested extensions.
///
/// Extensions are matched exactly and should not include a leading dot.
/// Ignore patterns are applied to both files and directories; ignored
/// directories are not traversed. The returned paths are sorted and
/// deduplicated.
///
/// # Arguments
///
/// * `entry_dirs` - Directories from which to begin recursive traversal.
/// * `all_exts` - File extensions to include, without leading dots.
/// * `ignore_list` - Patterns used to exclude matching files and directories.
///
/// # Errors
///
/// Returns an error if an ignore pattern is invalid or a directory entry
/// cannot be read.
#[napi(namespace = "cache")]
pub fn get_file_paths(
    entry_dir: PathBuf,
    all_exts: DefaultFiles,
    ignore_list: Option<Vec<String>>,
) -> napi::Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = Vec::new();
    let ignores = [all_exts.ignore_list, ignore_list.unwrap_or(vec![])].concat();
    let matcher =
        IgnoreMatcher::new(ignores).map_err(|error| napi::Error::from_reason(error.reason))?;
    let mut dirs = vec![entry_dir];
    while let Some(dir) = dirs.pop() {
        let root = if dir == PathBuf::from(".") {
            std::env::current_dir().unwrap()
        } else {
            dir
        };
        for entry in fs::read_dir(root)? {
            let entry = entry?;
            let path = entry.path();
            let file_type = entry.file_type()?;

            if file_type.is_dir() {
                if !matcher.is_ignored(path.to_string_lossy().to_string()) {
                    dirs.push(path);
                }
            } else if file_type.is_file() || (file_type.is_symlink() && path.is_file()) {
                if let Some(ext) = path.extension().and_then(|e| e.to_str())
                    && all_exts.all_exts.contains(&ext.to_string())
                    && !matcher.is_ignored(path.to_string_lossy().to_string())
                {
                    files.push(path);
                }
            }
        }
    }

    files.sort();
    files.dedup();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::get_file_paths;
    use crate::cache::helpers::def_files::default_files;
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            static NEXT_ID: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "bagan-core-assets-{}-{}",
                std::process::id(),
                NEXT_ID.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(path.join("_assets")).expect("create test assets directory");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).expect("remove temporary test directory");
        }
    }

    #[test]
    fn collects_images_scripts_and_stylesheets_from_assets_directory() {
        let root = TestDir::new();
        let asset_dir = root.path().join("_assets");
        let expected = ["site.css", "site.ico", "site.js"].map(|name| asset_dir.join(name));
        for path in &expected {
            fs::write(path, "").expect("create test asset");
        }

        let files = get_file_paths(root.path().to_path_buf(), default_files(), Some(vec![]))
            .expect("collect files");

        assert_eq!(files, expected);
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::collect::def_files::default_files;

//     use super::get_file_paths;
//     use std::{
//         fs,
//         path::{Path, PathBuf},
//         sync::atomic::{AtomicU64, Ordering},
//     };

//     struct TestDir(PathBuf);

//     impl TestDir {
//         fn new() -> Self {
//             static NEXT_ID: AtomicU64 = AtomicU64::new(0);
//             let path = std::env::temp_dir().join(format!(
//                 "bagan-core-get-files-{}-{}",
//                 std::process::id(),
//                 NEXT_ID.fetch_add(1, Ordering::Relaxed)
//             ));
//             fs::create_dir(&path).expect("create temporary test directory");
//             Self(path)
//         }

//         fn path(&self) -> &Path {
//             &self.0
//         }

//         fn write(&self, relative_path: &str) -> PathBuf {
//             let path = self.0.join(relative_path);
//             fs::create_dir_all(path.parent().expect("file path has a parent"))
//                 .expect("create test file parent directories");
//             fs::write(&path, "").expect("create test file");
//             path
//         }
//     }

//     impl Drop for TestDir {
//         fn drop(&mut self) {
//             fs::remove_dir_all(&self.0).expect("remove temporary test directory");
//         }
//     }

//     #[test]
//     fn collects_matching_files_recursively_sorted_and_deduplicated() {
//         let root = TestDir::new();
//         let nested_file = root.write("nested/a.md");
//         let root_file = root.write("z.md");
//         root.write("nested/ignored.txt");

//         let files = get_file_paths(
//             vec![root.path().to_path_buf(), root.path().join("nested")],
//             default_files(),
//             Some(vec![]),
//         )
//         .expect("collect files");

//         assert_eq!(files, vec![nested_file, root_file]);
//     }

//     #[test]
//     fn skips_ignored_directories_and_files() {
//         let root = TestDir::new();
//         let kept_file = root.write("public/keep.md");
//         root.write("private/hidden.md");
//         root.write("public/exclude.md");

//         let files = get_file_paths(
//             vec![root.path().to_path_buf()],
//             default_files(),
//             Some(vec!["private".to_string(), "exclude.md".to_string()]),
//         )
//         .expect("collect files");

//         assert_eq!(files, vec![kept_file]);
//     }

//     #[test]
//     fn returns_an_error_when_an_entry_directory_cannot_be_read() {
//         let root = TestDir::new();
//         let missing_dir = root.path().join("missing");

//         let result = get_file_paths(vec![missing_dir], default_files(), Some(vec![]));

//         assert!(result.is_err());
//     }
// }
