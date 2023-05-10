use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::{copy, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use indicatif::{ProgressBar, ProgressStyle};
use pulldown_cmark::{Event, Parser, Tag};
use regex::Regex;
use serde::Deserialize;
use walkdir::{DirEntry, WalkDir};

use crate::modules::notes::markdown;
use crate::modules::notes::markdown::{Page, PageLoader, WikilinkType};

pub fn get_pages(source: PathBuf) -> Vec<Page> {
    let files = markdown::get_markdown_files(source).ok().unwrap();
    let pathbufs = files.into_iter().map(|p| p.ok().unwrap()).collect::<Vec<PathBuf>>();
    let pages = pathbufs.into_iter().map(|path| { PageLoader::from_path(&path) }).collect::<Vec<Page>>();
    (&pages).into_iter().for_each(|page| println!("{:?}", page.wikilinks));
    return pages;
}

pub fn build_backlinks(pages: &[Page]) -> HashMap<String, Vec<(String, usize)>> {
    let mut backlinks: HashMap<String, Vec<(String, usize)>> = HashMap::new();

    for page in pages {
        for link in &page.wikilinks {
            // Skip non-TEXT links
            if link.link_type != WikilinkType::TEXT {
                continue;
            }

            let target_title = &link.link;

            // Skip empty links (i.e., just anchors)
            if target_title.is_empty() {
                continue;
            }

            let title = page.title.clone();

            let entry = backlinks.entry(target_title.clone()).or_insert_with(Vec::new);
            let existing_entry = entry.iter_mut().find(|(ref other_title, _)| **other_title == title);

            if let Some((_, ref mut count)) = existing_entry {
                *count += 1;
            } else {
                entry.push((title, 1));
            }
        }
    }

    backlinks
}

/// Build a site using Humble.
/// Reads markdown files from `source` and processes them into `destination`.
pub fn build(source: PathBuf, destination: PathBuf) -> (Vec<Page>, HashMap<String, Vec<(String, usize)>>) {
    let files = get_pages(source);
    println!("Filtering publishable");
    let pages = files.into_iter().filter(|page| {
        let mut lines = page.contents.split("\n");
        lines.any(|line| line.starts_with("publish: true"))
    }).collect::<Vec<Page>>();
    let backlinks = build_backlinks(&pages);
    println!("Backlinks: {:?}", backlinks);
    (pages, backlinks)
}

