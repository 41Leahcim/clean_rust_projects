#![warn(clippy::pedantic, clippy::nursery, clippy::restriction)]
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::blanket_clippy_restriction_lints,
    clippy::implicit_return,
    clippy::print_stdout,
    clippy::question_mark_used,
    clippy::use_debug,
    clippy::else_if_without_else,
    clippy::unwrap_used,
    clippy::print_stderr,
    clippy::min_ident_chars,
    clippy::allow_attributes_without_reason,
    clippy::arbitrary_source_item_ordering
)]

use std::{
    env::args,
    fs::{self, DirEntry},
    io,
    path::PathBuf,
};

use core::fmt::{self, Debug, Formatter};

struct FailedToRemoveError(PathBuf);

impl Debug for FailedToRemoveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Failed to remove directory: {:?}", self.0)
    }
}

struct RustProject {
    root: bool,
    target: Option<PathBuf>,
}

#[expect(clippy::single_call_fn)]
fn process_entry(entry: &DirEntry, too_explore: &mut Vec<PathBuf>, project: &mut RustProject) {
    // Take the metadata
    let Ok(metadata) = entry.metadata() else {
        return;
    };

    // Take the name of the file
    let file_name = entry.file_name();

    // If it's a Cargo.toml file, this directory is a project root
    if metadata.is_file() && file_name == "Cargo.toml" {
        project.root = true;
        return;
    }

    // Skip all other entries that aren't directory
    if !metadata.is_dir() {
        return;
    }

    // If the directory has the name "target", we probably want to delete it.
    // So we save it's path as the target directory.
    // Otherwise, it's another directory to search through
    if file_name == "target" {
        project.target = Some(entry.path());
    } else {
        too_explore.push(entry.path());
    }
}

fn remove_target_directories(path: PathBuf) -> Result<(), FailedToRemoveError> {
    // A buffer containing paths that still have to be explored
    let mut too_explore = vec![path];

    // Iterate through the paths
    while let Some(dir) = too_explore.pop() {
        // Create a new rust project object
        let mut project = RustProject {
            root: false,
            target: None,
        };

        // Try to read the directory entries
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };

        // Iterate through them, skip the errors
        for entry in entries.flatten() {
            // Process the current entry
            process_entry(&entry, &mut too_explore, &mut project);
        }

        // Remove the target directory if found in a project root.
        // If it's found anywhere else, we add it to the diretories to explore.
        if let Some(target) = project.target {
            if project.root {
                println!("{target:?}");
                fs::remove_dir_all(&target)
                    .map_err(|_error| FailedToRemoveError(target.clone()))?;
            } else {
                too_explore.push(target);
            }
        }
    }
    Ok(())
}

fn main() {
    // First iterate through the paths passed by argument
    for path in args().skip(1) {
        // Remove target directories
        remove_target_directories(path.into()).unwrap();
    }

    // Read a path from the screen if none was passed as argument
    if args().len() == 1 {
        // Ask for the path
        eprint!("Enter the path: ");

        // Read the path
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();

        // Remove target directories
        remove_target_directories(buffer.trim().to_owned().into()).unwrap();
    }
}
