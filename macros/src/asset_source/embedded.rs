use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use proc_macro::TokenStream;
use quote::quote;

pub fn expand_dir(path: &Path) -> TokenStream {
    let mut content = Content::new();
    content.push_dir(None, Id::new());

    read_dir(path, &mut content, Id::new());

    content.sort();

    content.to_token_stream()
}

fn extension_of(path: &Path) -> Option<&str> {
    match path.extension() {
        Some(ext) => ext.to_str(),
        None => Some(""),
    }
}

/// Read the directory recursively.
fn read_dir(path: &Path, content: &mut Content, id: Id) {
    let dir = path.read_dir().expect("Error reading assets directory");

    for elem in dir {
        let path = elem.expect("Error reading directory entry").path();

        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            let this_id = id.clone().push(stem);

            if path.is_dir() {
                content.push_dir(Some(&id), this_id.clone());
                read_dir(&path, content, this_id);
            } else if path.is_file() {
                if let Some(ext) = extension_of(&path) {
                    let ext = ext.to_owned();
                    let desc = FileDesc(this_id, ext, path);
                    content.push_file(desc, &id);
                }
            }
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Id(String);

impl Id {
    fn new() -> Id {
        Id(String::new())
    }

    fn push(mut self, id: &str) -> Id {
        if !self.0.is_empty() {
            self.0.push('.');
        }
        self.0.push_str(id);
        self
    }
}

struct FileDesc(Id, String, PathBuf);

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum DirEntry {
    File(Id, String),
    Dir(Id),
}

struct Content {
    files: Vec<FileDesc>,
    dirs: BTreeMap<Id, Vec<DirEntry>>,
}

impl Content {
    fn new() -> Content {
        Content {
            files: Vec::new(),
            dirs: BTreeMap::new(),
        }
    }

    /// Add a file in a directory.
    fn push_file(&mut self, desc: FileDesc, dir_id: &Id) {
        let entry = DirEntry::File(desc.0.clone(), desc.1.clone());
        self.dirs
            .get_mut(dir_id)
            .expect("File without directory")
            .push(entry);
        self.files.push(desc);
    }

    /// Add a sub directory.
    fn push_dir(&mut self, parent: Option<&Id>, id: Id) {
        if let Some(parent) = parent {
            let entry = DirEntry::Dir(id.clone());
            self.dirs
                .get_mut(parent)
                .expect("Directory without parent")
                .push(entry);
        }
        self.dirs.insert(id, Vec::new());
    }

    /// Sorts directory content to ensure reproducible builds.
    fn sort(&mut self) {
        // We can't use `sort_unstable_by_key` for some lifetime reason.
        self.files
            .sort_unstable_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));

        for dir in self.dirs.values_mut() {
            dir.sort_unstable();
        }
    }

    fn to_token_stream(&self) -> TokenStream {
        let files = self.files.iter().map(|FileDesc(Id(id), ext, path)| {
            let path = path.display().to_string();
            quote! {
                ((#id, #ext), (include_bytes!(#path) as &[u8]))
            }
        });

        let dirs = self.dirs.iter().map(|(Id(id), entries)| {
            let entries = entries.iter().map(|entry| match entry {
                DirEntry::File(Id(id), ext) => quote! {
                    pixel_game_lib::assets::source::DirEntry::File(#id, #ext)
                },
                DirEntry::Dir(Id(id)) => quote! {
                    pixel_game_lib::assets::source::DirEntry::Directory(#id)
                },
            });
            quote! {
                (#id, &[ #(#entries),* ])
            }
        });

        quote! {
            pixel_game_lib::assets::source::Embedded::from(
                pixel_game_lib::assets::source::RawEmbedded {
                    files: &[
                        #(#files),*
                    ],
                    dirs: &[
                        #(#dirs),*
                    ],
                }
            ).to_owned()
        }
        .into()
    }
}
