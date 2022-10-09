extern crate json;
extern crate reqwest;

use std::{str::FromStr};

use json::JsonValue;
use JsonValue::Null;

#[derive(Debug)]
pub struct Empty;

pub static mut ELEMENTS_COUNT: u128 = 0;

//TODO: add better debug formatting
#[derive(PartialEq)]
pub struct Element {
    author: String,
    data: String,
    kind: String,
    post_hint: String,
    url: String, //url_overridden_by_dest
    ups: usize,
    children: Vec<Element>,
    created_utc: String,
    depth: String,
}

impl std::fmt::Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut children = String::new();

        for child in &self.children {
            children += &format!("{:?}", child);
        }

        if !self.data.is_empty()|| !self.author.is_empty() {
            let indent_char = " ";
            //let secondary_indent_char = " ";
            let indent = indent_char.repeat(usize::from_str(&self.depth).unwrap_or(0));
            let ups_indent = indent_char.repeat(self.ups.to_string().len());
            //TODO: make this more readable
            return f.write_fmt(format_args!(
                "{}{} {} {}: {}\n{}",
                indent,
                self.depth,
                self.ups,
                self.author,
                self.data.replace(
                    '\n',
                    &(String::from('\n')
                        + &(indent.to_string() + &indent_char.repeat(self.author.len() + 4) + &ups_indent + " ")) //.replace(indent_char, secondary_indent_char))
                ),
                children
            ));
        }
        Ok(())
    }
}

impl Element {
    pub fn create(data: &JsonValue) -> Result<Element, Empty> {
        if *data == JsonValue::Null {
            return Err(Empty {});
        }
        unsafe {
            ELEMENTS_COUNT += 1;
        }
        Ok(Element {
            author: match Element::get_data(data, "author") {
                Ok(o) => o,
                _ => String::new(),
            },
            data: match Element::get_data(data, "body") {
                Ok(o) => o,
                _ => match Element::get_data(data, "title") {
                    Ok(o) => o,
                    _ => match Element::get_data(data, "selftext") {
                        Ok(o) => o,
                        _ => String::new(),
                    },
                },
            },
            children: match Element::get_replies(data) {
                Ok(o) => o,
                _ => Vec::new(),
            },
            ups: match Element::get_data(data, "ups") {
                Ok(o) => match o.parse::<usize>() {
                    Ok(o) => o,
                    _ => 0,
                },
                _ => 0,
            },
            post_hint: match Element::get_data(data, "post_hint") {
                Ok(o) => o,
                _ => String::new(),
            },
            url: match Element::get_data(data, "url_overridden_by_dest") {
                Ok(o) => o,
                _ => String::new(),
            },
            //a hacky way, but "kind" attribute is higher in the json tree so it would be a pain in the butt to get it that way
            kind: match Element::get_data(data, "name") {
                Ok(o) => o[0..2].to_owned(),
                _ => String::new(),
            },
            created_utc: match Element::get_data(data, "created_utc") {
                Ok(o) => o,
                _ => String::new(),
            },
            depth: match Element::get_data(data, "depth") {
                //TODO: Represent depth as a number
                Ok(o) => o,
                _ => String::from("0"),
            },
        })
    }

    pub fn empty() -> Element {
        Element {
            author: String::new(),
            data: String::new(),
            children: Vec::<Element>::new(),
            ups: 0,
            post_hint: String::new(),
            url: String::new(),
            kind: String::new(),
            created_utc: String::new(),
            depth: String::new(),
        }
    }

    pub fn init(data: &JsonValue) -> Vec<Element>{
        let mut elements = Vec::<Element>::new();

        for x in data.members() {
            for y in x["data"]["children"].members() {
                let data = y["data"].clone();
    
                match Element::create(&data) {
                    Ok(o) => elements.push(o),
                    _ => elements.push(Element::empty()),
                }
            }
        }
        elements
    }

    fn get_replies(element: &JsonValue) -> Result<Vec<Element>, Empty> {
        let mut out = Vec::<Element>::new();
        if element["replies"] != Null {
            for r in element["replies"]["data"]["children"].members() {
                let element = match Element::create(&r["data"]) {
                    Ok(o) => o,
                    _ => continue,
                };
    
                out.push(element);
            }
        }
        if !out.is_empty() {
            Ok(out)
        } else {
            Err(Empty {})
        }
    }
    
    fn get_data(element: &JsonValue, field: &str) -> Result<String, Empty> {
        if element[field] != JsonValue::Null {
            return Ok(element[field].to_string());
        }
        Err(Empty {})
    }
}