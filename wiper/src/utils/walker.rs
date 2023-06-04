use std::ffi::OsStr;

use walkdir::WalkDir;

pub fn count_and_size(path: impl AsRef<std::path::Path>) -> (usize, u64) {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|entry| entry.metadata().ok())
        .filter(|metadata| metadata.is_file())
        .fold((0, 0), |(file_count, size_sum), metadata| {
            (file_count + 1, size_sum + metadata.len())
        })
}

pub fn is_node_modules(file_name: &OsStr) -> bool {
    file_name.to_string_lossy() == "node_modules"
}

pub fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

pub fn filter_entry_parent_from_predicate<P>(entry: &walkdir::DirEntry, filter_filename_predicate: &P) -> bool
where
    P: Fn(&OsStr) -> bool,
    {
    entry.path().parent().map_or(true, |parent_path| {
        parent_path
            .file_name()
            .map_or(true, |file_name| !filter_filename_predicate(file_name))
    })
}

pub fn get_dir_list_from_path<'a, P, F>(
    path: &P,
    filter_filename_predicate: &'a F,
) -> impl Iterator<Item = walkdir::DirEntry> + 'a
where
    P: AsRef<std::path::Path>,
    F: Fn(&OsStr) -> bool,
{
    WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_entry(|entry| filter_entry_parent_from_predicate(entry, filter_filename_predicate))
        .filter_map(|e| e.ok())
        // .filter(|entry| entry.file_type().is_dir())
        .filter(|entry| filter_filename_predicate(entry.file_name()))
}


pub fn delete_entries(entries: &[walkdir::DirEntry]) -> Result<(), std::io::Error> {
    // for entry in entries {
    //     if entry.file_type().is_dir() {
    //         std::fs::remove_dir_all(entry.path())?;
    //     } else {
    //         std::fs::remove_file(entry.path())?;
    //     }
    // }
    Ok(())
}