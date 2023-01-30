extern crate json;
extern crate reqwest;

use std::{str::FromStr};

use json::JsonValue;
use JsonValue::Null;

#[derive(Debug)]
pub struct Empty;

pub static mut ELEMENTS_COUNT: u128 = 0;

//var,field name, def value
macro_rules! get_data_wrapper{
    ($var:ident,$name:expr,$def_value:expr)=>{
        match Element::get_data($var,stringify!($name)){
            Ok(o)=>o,
            _ => $def_value
        }
    }
}

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
    permalink: String,
}

impl std::fmt::Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let children = self.children.iter().map(|x| format!("{x:?}")).collect::<String>();

        if !self.data.is_empty() || !self.author.is_empty() {
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
                        + &(indent.to_string()
                            + &indent_char.repeat(self.author.len() + 4)
                            + &ups_indent
                            + " ")) //.replace(indent_char, secondary_indent_char))
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
        let mut total_data = String::new();

        let mut add_to_total = |var : String|{
            if !var.is_empty(){
                if !total_data.is_empty(){
                    total_data += "\n";
                }
                total_data += &var;
            } 
        };

        let _title = get_data_wrapper!(data,title,String::new());
        let selftext = get_data_wrapper!(data,selftext,String::new());
        let body = get_data_wrapper!(data,body,String::new());
        add_to_total(_title);
        add_to_total(selftext);
        add_to_total(body);
        Ok(Element {
            author : get_data_wrapper!(data,author,String::new()),
            data:total_data,
            children: match Element::get_replies(data) {
                Ok(o) => o,
                _ => Vec::new(),
            },
            ups: match get_data_wrapper!(data,ups,"0".to_string()).parse::<usize>(){
                Ok(o)=>o,
                _=>0usize
            },
            post_hint: get_data_wrapper!(data, "post_hint", String::new()),
            url : get_data_wrapper!(data,url_overridden_by_dest,String::new()),
            //a hacky way, but "kind" attribute is higher in the json tree so it would be a pain in the butt to get it that way
            kind: get_data_wrapper!(data,name,String::new())[0..2].to_owned(),
            created_utc: get_data_wrapper!(data, "created_utc",String::new()),
            depth: get_data_wrapper!(data,depth,"0".to_string()),
            permalink: get_data_wrapper!(data,permalink,String::new())
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
            permalink: String::new(),
        }
    }

    pub fn init(data: &JsonValue) -> Vec<Element> {
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
            return Ok(out)
        } 
        Err(Empty {})
    }

    fn get_data(element: &JsonValue, field: &str) -> Result<String, Empty> {
        if element[field] != JsonValue::Null {
            return Ok(element[field].to_string());
        }
        Err(Empty {})
    }
}
