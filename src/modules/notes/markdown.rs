use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use globmatch::Matcher;
use lazy_static::lazy_static;
use regex::Regex;
use rlua::{Context, Lua, Table, ToLua, ToLuaMulti};

use crate::modules::core::get_files;
use crate::modules::notes::markdown::WikilinkType::TEXT;

lazy_static! {
    static ref WIKILINK_REGEX: Regex = Regex::new(r"(!)?\[\[(.*?)\]\]").unwrap();
    static ref MEDIA_REGEX: Regex = Regex::new(r"!\[(.*)?\]\((.*)\)").unwrap();
    static ref IMAGE_EXTENSIONS: Vec<String> = vec!["jpg".to_string(),
        "png".to_string(), "svg".to_string(), "jpeg".to_string(), "gif".to_string()];
}

#[derive(Debug)]
pub struct Page {
    pub title: String,
    pub path: PathBuf,
    pub contents: String,
    pub wikilinks: Vec<WikiLink>,
}


pub trait PageLoader {
    fn from_path(path: &PathBuf) -> Self;
    fn title_from_path(path: &PathBuf) -> String;
}

impl PageLoader for Page {
    fn from_path(path: &PathBuf) -> Self {
        // Read the contents, extract wikilinks, or perform other initializations here...
        let contents = fs::read_to_string(path).ok().unwrap();
        Self {
            path: path.clone(),
            title: Self::title_from_path(path),
            contents: contents.clone(),
            wikilinks: extract_links(&contents),
        }
    }

    fn title_from_path(path: &PathBuf) -> String {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("")
            .to_string()
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum WikilinkType {
    IMAGE,
    TEXT,
}

#[derive(Debug)]
pub struct WikiLink {
    pub name: String,
    pub link: String,
    pub anchor: String,
    pub link_type: WikilinkType,
    pub original: String,
}

pub fn get_markdown_files<'a>(root: PathBuf) -> Result<Matcher<'a, PathBuf>, String> {
    return get_files(root, &"**/*.md");
}

pub fn extract_links(contents: &str) -> Vec<WikiLink> {
    return WIKILINK_REGEX.captures_iter(contents)
        .map(|captures| parse_wikilink(captures.get(2).unwrap().as_str()))
        .filter(|wikilink| wikilink.as_ref().ok().is_some())
        .map(|wikilink| wikilink.unwrap())
        .collect::<Vec<WikiLink>>();
}

impl FromStr for WikiLink {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let link_contents = s
            .trim_start_matches("[[")
            .trim_end_matches("]]");

        let (link, name) = if let Some(pipe_pos) = link_contents.find('|') {
            let (link, name) = link_contents.split_at(pipe_pos);
            (link.trim(), name.trim_start_matches('|'))
        } else {
            (link_contents, link_contents)
        };

        let (link_without_anchor, anchor) = if let Some(anchor_pos) = link.find('#') {
            let (link, anchor) = link.split_at(anchor_pos);
            (link.trim(), anchor.trim_start_matches('#'))
        } else {
            (link, "")
        };

        let is_just_anchor = link.starts_with('#');
        let final_link = if is_just_anchor { "" } else { link_without_anchor };
        let final_anchor = if is_just_anchor { link_contents.trim_start_matches('#') } else { anchor };
        let final_name = if name == link { final_link.to_string() } else { name.to_string() };

        let link_type = filename_type(final_link);

        Ok(WikiLink {
            name: final_name,
            link: String::from(final_link),
            anchor: String::from(final_anchor),
            link_type,
            original: "".to_string(),
        })
    }
}

pub fn parse_wikilink(s: &str) -> Result<WikiLink, Box<dyn Error>> {
    WikiLink::from_str(s)
}

pub fn filename_type(filename: &str) -> WikilinkType {
    let lower_ext = filename.split('.').last().unwrap_or("").to_lowercase();
    match lower_ext.as_str() {
        "jpg" | "jpeg" | "png" | "gif" => WikilinkType::IMAGE,
        _ => WikilinkType::TEXT,
    }
}

#[cfg(test)]
mod tests {
    use crate::modules::notes::markdown::WikilinkType::IMAGE;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_simple() {
        let wikilink = parse_wikilink("[[Test]]").unwrap();
        assert_eq!(wikilink.name, "Test");
        assert_eq!(wikilink.link, "Test");
        assert_eq!(wikilink.anchor, "");
        assert_eq!(wikilink.link_type, TEXT);
    }

    #[test]
    fn test_alternate_name() {
        let wikilink = parse_wikilink("[[Test|Another]]").unwrap();
        assert_eq!(wikilink.name, "Another");
        assert_eq!(wikilink.link, "Test");
        assert_eq!(wikilink.anchor, "");
        assert_eq!(wikilink.link_type, TEXT);
    }

    #[test]
    fn test_anchor() {
        let wikilink = parse_wikilink("[[Test#Anchor]]").unwrap();
        assert_eq!(wikilink.name, "Test");
        assert_eq!(wikilink.link, "Test");
        assert_eq!(wikilink.anchor, "Anchor");
        assert_eq!(wikilink.link_type, TEXT);
    }

    #[test]
    fn test_just_anchor() {
        // Just the anchor
        let wikilink = parse_wikilink("[[#Anchor Test]]").unwrap();
        assert_eq!(wikilink.name, "");
        assert_eq!(wikilink.link, "");
        assert_eq!(wikilink.anchor, "Anchor Test");
        assert_eq!(wikilink.link_type, TEXT);
    }

    #[test]
    fn test_anchor_and_name() {
        let wikilink = parse_wikilink("[[Test#Anchor2|Another]]").unwrap();
        assert_eq!(wikilink.name, "Another");
        assert_eq!(wikilink.link, "Test");
        assert_eq!(wikilink.anchor, "Anchor2");
        assert_eq!(wikilink.link_type, TEXT);
    }

    #[test]
    fn test_image() {
        let wikilink = parse_wikilink("[[Test.jpg]]").unwrap();
        assert_eq!(wikilink.name, "Test.jpg");
        assert_eq!(wikilink.link, "Test.jpg");
        assert_eq!(wikilink.anchor, "");
        assert_eq!(wikilink.link_type, IMAGE);
    }

    #[test]
    fn test_image_name() {
        let wikilink = parse_wikilink("[[Test.png|Another]]").unwrap();
        assert_eq!(wikilink.name, "Another");
        assert_eq!(wikilink.link, "Test.png");
        assert_eq!(wikilink.anchor, "");
        assert_eq!(wikilink.link_type, IMAGE);
    }

    #[test]
    fn test_extension_jpg() {
        let filename = "foo.jpg";
        assert_eq!(filename_type(filename), IMAGE);
        let filename2 = "FOO.JPG";
        assert_eq!(filename_type(filename2), IMAGE);
    }

    #[test]
    fn test_extension_png() {
        let filename = "foo.png";
        assert_eq!(filename_type(filename), IMAGE);
        let filename2 = "FOO.PNG";
        assert_eq!(filename_type(filename2), IMAGE);
    }

    #[test]
    fn test_extension_gif() {
        let filename = "foo.gif";
        assert_eq!(filename_type(filename), IMAGE);
        let filename2 = "FOO.GIF";
        assert_eq!(filename_type(filename), IMAGE);
    }
}