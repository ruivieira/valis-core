use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
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
    static ref IMAGE_EXTENSIONS: Vec<String> = vec![
        "jpg".to_string(),
        "png".to_string(),
        "svg".to_string(),
        "jpeg".to_string(),
        "gif".to_string()
    ];
}

#[derive(Debug)]
pub struct Page {
    pub title: String,
    pub path: PathBuf,
    pub contents: String,
    pub wikilinks: Vec<WikiLink>,
}

impl Clone for Page {
    fn clone(&self) -> Self {
        Page {
            title: self.title.clone(),
            path: self.path.clone(),
            contents: self.contents.clone(),
            wikilinks: self.wikilinks.clone(),
        }
    }
}

fn convert_wikilinks_to_hugo(contents: &str) -> String {
    // Regular expression to match wikilinks
    let re = Regex::new(r"(!?)\[\[(.*?)\]\]").unwrap();

    // Function to convert a matched wikilink to Hugo format
    let replace_func = |caps: &regex::Captures| {
        let is_image = &caps[1] == "!";
        let link_and_name = &caps[2];

        // Check if the matched link is an image
        if is_image {
            format!(
                "{{{{< figure src=\"/assets/{}\" alt=\"{}\" >}}}}",
                link_and_name, link_and_name
            )
        }
        // Check if the matched link contains a '|'
        else if let Some(pipe_index) = link_and_name.find('|') {
            // If it does, split the string on this character to get the link and the alternate name
            let (link, name) = link_and_name.split_at(pipe_index);
            let name = &name[1..]; // Remove the leading '|'
            format!("[{}]({{{{< ref \"{}\" >}}}})", name, link)
        } else {
            // If there's no '|', use the entire match as both the link and the name
            format!("[{}]({{{{< ref \"{}\" >}}}})", link_and_name, link_and_name)
        }
    };

    // Replace all matches in the contents
    re.replace_all(contents, replace_func).into_owned()
}

impl Page {
    pub fn save_to_file(&self, directory: &PathBuf) -> std::io::Result<()> {
        // Construct the full file path
        let mut file_path = directory.clone();
        file_path.push(format!("{}.md", &self.title));

        // Open a file in write-only mode
        let mut file = File::create(&file_path)?;

        // Convert wikilinks to Hugo format
        let contents = convert_wikilinks_to_hugo(&self.contents);

        // Write the contents to file
        file.write_all(contents.as_bytes())
    }
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

#[derive(Debug, PartialEq, Copy, Clone)]
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

impl Clone for WikiLink {
    fn clone(&self) -> Self {
        WikiLink {
            name: self.name.to_string(),
            link: self.link.to_string(),
            anchor: self.anchor.to_string(),
            link_type: self.link_type.clone(),
            original: self.original.to_string(),
        }
    }
}

pub fn get_markdown_files<'a>(root: PathBuf) -> Result<Matcher<'a, PathBuf>, String> {
    return get_files(root, &"**/*.md");
}

pub fn remove_code_blocks(s: &str) -> String {
// Matches code blocks with or without language identifiers
    let re = Regex::new(r"```.*?```").unwrap();
    re.replace_all(s, "").to_string()
}


pub fn extract_links(contents: &str) -> Vec<WikiLink> {
    return WIKILINK_REGEX
        .captures_iter(&remove_code_blocks(contents))
        .map(|captures| parse_wikilink(captures.get(2).unwrap().as_str()))
        .filter(|wikilink| wikilink.as_ref().ok().is_some())
        .map(|wikilink| wikilink.unwrap())
        .collect::<Vec<WikiLink>>();
}

impl FromStr for WikiLink {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let link_contents = s.trim_start_matches("[[").trim_end_matches("]]");

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
        let final_link = if is_just_anchor {
            ""
        } else {
            link_without_anchor
        };
        let final_anchor = if is_just_anchor {
            link_contents.trim_start_matches('#')
        } else {
            anchor
        };
        let final_name = if name == link {
            final_link.to_string()
        } else {
            name.to_string()
        };

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
