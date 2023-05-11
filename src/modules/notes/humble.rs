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

fn replace_first(input: &str, from: &str, to: &str) -> String {
    let start_index = match input.find(from) {
        Some(index) => index,
        None => return input.to_string(),
    };
    let end_index = start_index + from.len();
    [&input[..start_index], to, &input[end_index..]].concat()
}

fn add_backlinks(page: &mut Page, backlinks: &HashMap<String, Vec<(String, usize)>>) -> Result<(), Box<dyn std::error::Error>> {
    let empty = "backlinks: []\nbacklinks_count: []\n".to_string();

    // Check if this page has any backlinks
    if let Some(links) = backlinks.get(&page.title) {
        // Convert the Vec<(String, usize)> into separate Vec<String> and Vec<usize>
        let (backlink_titles, backlink_counts): (Vec<String>, Vec<usize>) = links.iter().map(|(title, count)| (title.clone(), *count)).unzip();

        // Convert the Vec<String> and Vec<usize> into a single String each, separated by commas
        let backlink_titles_string = backlink_titles.into_iter().map(|title| format!("\"{}\"", title)).collect::<Vec<String>>().join(", ");
        let backlink_counts_string = backlink_counts.iter().map(|count| count.to_string()).collect::<Vec<_>>().join(", ");

        // Find the end of the first "---\n"
        let front_matter_end = page.contents.find("---\n").unwrap() + 4;

        // Insert the backlinks and backlinks_count at the beginning of the page content
        page.contents.insert_str(front_matter_end, &format!("backlinks: [{}]\n", backlink_titles_string));
        page.contents.insert_str(front_matter_end + backlink_titles_string.len() + 14, &format!("backlinks_count: [{}]\n", backlink_counts_string));
    } else {
        let contents = replace_first(&page.contents, "---", &format!("---\n{}", empty));
        page.contents = contents;
    }

    Ok(())
}

fn save_pages_to_files(pages: &[Page], dest: &PathBuf) -> std::io::Result<()> {
    // Create the contents subdirectory if it doesn't exist
    let mut contents_dir = dest.clone();
    contents_dir.push("posts");
    fs::create_dir_all(&contents_dir)?;

    for page in pages {
        if page.title == "Index" {
            // Save the Index page directly to dest
            page.title == "_index";
            page.save_to_file(&dest)?;
        } else {
            // Save other pages to the contents subdirectory
            page.save_to_file(&contents_dir)?;
        }
    }

    Ok(())
}

fn build_image_map(dir: &PathBuf, map: &mut HashMap<String, PathBuf>) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                build_image_map(&path, map)?;
            } else if let Some(extension) = path.extension() {
                if ["png", "jpg", "gif", "svg"].contains(&extension.to_str().unwrap()) {
                    if let Some(filename) = path.file_name() {
                        map.insert(filename.to_str().unwrap().to_string(), path.clone());
                    }
                }
            }
        }
    }
    Ok(())
}

fn create_image_map(source_dir: &str) -> Result<HashMap<String, PathBuf>, std::io::Error> {
    let mut image_map: HashMap<String, PathBuf> = HashMap::new();
    let source_path = PathBuf::from(source_dir);
    build_image_map(&source_path, &mut image_map)?;
    Ok(image_map)
}

fn copy_images_from_page(page: &Page, image_map: &HashMap<String, PathBuf>, destination: &str) -> std::io::Result<()> {
    let image_link_pattern = Regex::new(r"!\[\[(.*?)\]\]").unwrap();
    let image_links: Vec<String> = image_link_pattern.captures_iter(&page.contents)
        .filter_map(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .collect();

    for link in image_links {
        if let Some(image_path) = image_map.get(&link) {
            let destination_path = Path::new(destination).join(&link);
            if let Some(parent_dir) = destination_path.parent() {
                fs::create_dir_all(parent_dir)?; // create all directories in the path if they don't exist
            }
            fs::copy(image_path, &destination_path)?;
        }
    }
    Ok(())
}

/// Build a site using Humble.
/// Reads markdown files from `source` and processes them into `destination`.
pub fn build(source: PathBuf, destination: PathBuf, assets: PathBuf) -> (Vec<Page>, HashMap<String, Vec<(String, usize)>>) {
    let search_markdown_spinner = ProgressBar::new_spinner();
    search_markdown_spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner} Searching for Markdown files..."),
    );
    search_markdown_spinner.enable_steady_tick(100);

    let files = get_pages(source.clone());

    search_markdown_spinner.finish_with_message("Finished searching for Markdown files.");

    let filter_publishable_spinner = ProgressBar::new_spinner();
    filter_publishable_spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner} Filtering publishable..."),
    );
    filter_publishable_spinner.enable_steady_tick(100);

    let pages = files.into_iter().filter(|page| {
        let mut lines = page.contents.split("\n");
        lines.any(|line| line.starts_with("publish: true"))
    }).collect::<Vec<Page>>();

    filter_publishable_spinner.finish_with_message("Finished filtering publishable.");

    let backlinks_spinner = ProgressBar::new_spinner();
    backlinks_spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner} Searching backlinks..."),
    );
    backlinks_spinner.enable_steady_tick(100);


    let backlinks = build_backlinks(&pages);

    backlinks_spinner.finish_with_message("Finished searching publishable.");

    let updated_pages = pages.into_iter().map(|mut page| {
        add_backlinks(&mut page, &backlinks).ok().unwrap();
        page
    }).collect::<Vec<Page>>();

    save_pages_to_files(&updated_pages, &destination).ok().unwrap();

    let copy_images_spinner = ProgressBar::new_spinner();
    copy_images_spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner} Copying images..."),
    );
    copy_images_spinner.enable_steady_tick(100);

    let image_map = create_image_map(source.clone().to_str().unwrap()).ok().unwrap();

    let saved_pages = updated_pages.into_iter().map(|page| {
        copy_images_from_page(&page, &image_map, assets.to_str().unwrap()).ok().unwrap();
        page
    }).collect::<Vec<Page>>();

    copy_images_spinner.finish_with_message("Finished copying images.");
    (saved_pages, backlinks)
}

