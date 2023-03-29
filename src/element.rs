extern crate json;
extern crate reqwest;

use std::{
    str::FromStr,
    sync::{Arc, Mutex},
    thread,
    time::SystemTime,
};

use json::JsonValue;

use crate::{convert_time, request};
use std::io::Write;

#[derive(Debug)]
pub struct Empty;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Default,
    HTML,
    JSON,
}

pub static mut NUM_COMMENTS: usize = 0;
pub static mut ELEMENTS_COUNT: usize = 1;
pub static mut MORE_ELEMENTS_COUNT: usize = 0;
pub static mut MORE_ELEMENTS: Vec<String> = Vec::new();
pub static mut FORMAT: Format = Format::Default;

macro_rules! get_safe {
    ($var:ident) => {
        unsafe { $var.clone() }
    };
}

//var,field name, def value
macro_rules! get_data_wrapper {
    ($var:ident,$name:expr,$def_value:expr) => {
        match Element::get_data($var, stringify!($name)) {
            Ok(o) => o,
            _ => $def_value,
        }
    };
}

//TODO: add better debug formatting
#[derive(Clone)]
pub struct Element {
    pub author: String,
    data: String,
    kind: String,
    url: String, //url_overridden_by_dest
    pub ups: usize,
    pub children: Vec<Element>,
    depth: String,
    permalink: String,
    pub id: String,
    pub parent_id: String,
    over_18: bool,
    pub created: usize,
    pub edited: usize,
}

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl std::fmt::Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.data.is_empty() || self.author.is_empty() {
            return std::fmt::Result::Err(std::fmt::Error::default());
        }
        return match get_safe!(FORMAT) {
            Format::Default => {
                let children = self
                    .children
                    .iter()
                    .map(|x| format!("{x:?}"))
                    .collect::<String>();

                let indent_char = " ";
                //let secondary_indent_char = " ";
                let indent = indent_char.repeat(usize::from_str(&self.depth).unwrap_or(0));
                let ups_indent = indent_char.repeat(self.ups.to_string().len());
                //TODO: make this more readable
                f.write_fmt(format_args!(
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
                ))
            }
            Format::HTML => {
                let children = self
                    .children
                    .iter()
                    .map(|x| format!("{x:?}"))
                    .collect::<String>();
                let indent_char = " ";
                let indent = "\t".to_owned()
                    + &indent_char.repeat(usize::from_str(&self.depth).unwrap_or(0));
                let url = if self.url.is_empty() {
                    String::new()
                } else {
                    format!("<a href=\"{}\">{}</a>", self.url, self.url)
                };
                let mut children_string = String::new();
                if !children.is_empty() {
                    children_string = format!("<ul>{children}</ul>");
                }
                let href = String::from("https://reddit.com") + &self.permalink;
                let author = self.author.clone();
                let ups = self.ups;
                let span_data = self.data.strip_prefix(&self.url).unwrap();
                f.write_fmt(format_args!(
                    "\n{indent}<div class=\"element\">
                    {indent}<h4><a href=\"{href}\">{author}</a> ⬆️{ups}:</h4>
                    {url}
                    <span>{span_data}</span>
                    {children_string}
                    \n{indent}</div>", //TODO: add human readable formatting
                ))
            }
            Format::JSON => {
                fn parse_json_element(elem: &Element) -> json::object::Object {
                    let mut json_object = json::object::Object::new();
                    json_object.insert("author", JsonValue::String(elem.author.clone()));
                    json_object.insert("data", JsonValue::String(elem.data.clone()));
                    json_object.insert("ups", JsonValue::from(elem.ups));
                    json_object.insert("edited", JsonValue::from(elem.edited));
                    json_object.insert("depth", JsonValue::String(elem.depth.to_string()));
                    json_object.insert("id", JsonValue::String(elem.id.clone()));
                    json_object.insert("kind", JsonValue::String(elem.kind.clone()));
                    json_object.insert("permalink", JsonValue::String(elem.permalink.clone()));
                    //json_object.insert("post_hint",JsonValue::String(elem.post_hint.clone()));
                    json_object.insert("url", JsonValue::String(elem.url.clone()));
                    json_object.insert("over_18", JsonValue::from(elem.over_18));
                    for child in &elem.children {
                        json_object.insert("children", json::from(parse_json_element(child)));
                    }
                    json_object
                }
                let json_object = parse_json_element(self);
                f.write_fmt(format_args!("{},\n", json_object.pretty(4)))
            }
        };
    }
}

impl Element {
    pub fn create(child: &JsonValue, max_elements: usize) -> Result<Self, Empty> {
        if get_safe!(ELEMENTS_COUNT) >= max_elements {
            return Err(Empty {});
        }
        let data = &child["data"];
        if *data == JsonValue::Null {
            return Err(Empty {});
        }
        //If the element lists more elements(it's kind is more)
        if child["kind"].clone() == "more" {
            unsafe {
                MORE_ELEMENTS_COUNT += data["count"].as_usize().map_or(0, |o| o)
            }
            for more_element in data["children"].members() {
                unsafe {
                    MORE_ELEMENTS.push(more_element.to_string());
                }
            }

            //Not really an error but it's the best way
            return Err(Empty {});
        }

        let mut total_data = String::new();

        let mut add_to_total = |var: String| {
            if !var.is_empty() {
                if !total_data.is_empty() {
                    total_data += "\n";
                }
                total_data += &var;
            }
        };

        let n_comments = get_data_wrapper!(data, num_comments, String::new());
        if !n_comments.is_empty() {
            unsafe {
                NUM_COMMENTS = n_comments.parse::<usize>().map_or(0,|o| o)
            }
        }

        let title_ = get_data_wrapper!(data, title, String::new());
        let url = get_data_wrapper!(data, url, String::new());
        let selftext = get_data_wrapper!(data, selftext, String::new());
        let body = get_data_wrapper!(data, body, String::new());
        add_to_total(url);
        add_to_total(title_);
        add_to_total(selftext);
        add_to_total(body);

        let author = get_data_wrapper!(data, author, String::new());

        if !(total_data.trim() == "[deleted]"
            && total_data.trim() == "[removed]"
            && author.trim() == "[deleted]"
            && author.trim() == "[removed]")
        {
            unsafe {
                ELEMENTS_COUNT += 1;
            }
        }
        Ok(
            //Use this for clarity
            #[allow(clippy::redundant_field_names)]
            Self {
                author,
                data: total_data,
                children: Element::get_replies(data, max_elements).map_or_else(|_| Vec::new(), |o| o),
                ups :get_data_wrapper!(data, ups, "0".to_string()).parse::<usize>().map_or(0usize, |o| o),
                //post_hint: get_data_wrapper!(data, "post_hint", String::new()),
                url: get_data_wrapper!(data, url_overridden_by_dest, String::new()),
                //a hacky way, but "kind" attribute is higher in the json tree so it would be a pain in the butt to get it that way
                kind: get_data_wrapper!(data, name, String::new())[0..2].to_owned(),
                //edited: get_data_wrapper!(data, edited, String::from("false")) == *"true",
                depth: get_data_wrapper!(data, depth, "0".to_string()),
                permalink: get_data_wrapper!(data, permalink, String::new()),
                id: get_data_wrapper!(data, id, String::new()),
                parent_id: {
                    let mut pid = get_data_wrapper!(data, parent_id, String::new());
                    if pid.len() > 3 {
                        pid = pid[3..].to_string();
                    }
                    pid
                },
                over_18: get_data_wrapper!(data, over_18, String::from("false")) == *"true",
                created: get_data_wrapper!(data, created, usize::MAX.to_string())
                .parse::<f32>().map_or(usize::MAX, |o| o as usize),
                edited: get_data_wrapper!(data, edited, usize::MAX.to_string()).parse::<f32>().map_or(usize::MAX, |o| o as usize),
            },
        )
    }

    pub fn init(data: &JsonValue, max_elements: usize) -> Vec<Self> {
        let mut elements = Vec::<Element>::new();

        for member in data.members() {
            for child in member["data"]["children"].members() {
                if get_safe!(ELEMENTS_COUNT) >= max_elements {
                    break;
                }
                //.If created element isn't empty (Ok) push it.
                if let Ok(o) = Element::create(child, max_elements) {
                    elements.push(o);
                }
            }
        }
        elements
    }

    fn get_replies(element: &JsonValue, max_elements: usize) -> Result<Vec<Self>, Empty> {
        let mut out = Vec::<Element>::new();
        if element["replies"] != JsonValue::Null {
            for child in element["replies"]["data"]["children"].members() {
                if get_safe!(ELEMENTS_COUNT) >= max_elements {
                    break;
                }
                let Ok(element) = Element::create(child, max_elements) else {continue};

                out.push(element);
            }
        }
        if !out.is_empty() {
            return Ok(out);
        }
        Err(Empty {})
    }

    fn get_data(element: &JsonValue, field: &str) -> Result<String, Empty> {
        if element[field] != JsonValue::Null {
            return Ok(element[field].to_string());
        }
        Err(Empty {})
    }

    //Recursively check every element, and if the first element matches appedn to it
    fn append_element(x: &mut Vec<Self>, y: &mut Vec<Self>) {
        for element in &mut *y {
            if x.is_empty() {
                break;
            }
            if *element == x[0] {
                x.remove(0);
                unsafe {
                    ELEMENTS_COUNT -= 1;
                }
                element.children.append(x);
                break;
            }
        }
        for e in y {
            Self::append_element(x, &mut e.children);
        }
    }

    pub async fn get_more_element(
        base_url: String,
        idx: Arc<Mutex<f64>>,
        more_start: SystemTime,
        last_line_length: Arc<Mutex<usize>>,
        elements: Arc<Mutex<Vec<Self>>>,
        max_comments: usize,
    ) -> Option<String> {
        if get_safe!(ELEMENTS_COUNT) >= max_comments {
            return None;
        }
        //build the url
        let url = base_url;
        let Ok(res) = request(url, None).await else{
            return None;
        };

        let Ok(data) = res.text().await else { todo!() };

        let json_data = match json::parse(&data) {
            Ok(o) => o,
            Err(e) => {
                println!("Failed to parse json data with error: {e}");
                std::process::exit(0);
            }
        };

        //Parse json data to elements
        let mut e = Element::init(&json_data, max_comments);

        {
            let idx_ = *idx.lock().unwrap();
            let last_line_length_ = *last_line_length.lock().unwrap();

            //calculate % of progress as a 64bit float 64
            let precent = idx_ / (get_safe!(MORE_ELEMENTS).len() as f64) * 100f64;

            //get time passed since start of getting 'more' elements
            let passed = std::time::SystemTime::now()
                .duration_since(more_start)
                .unwrap()
                .as_millis();

            //Get estimated time
            let eta = get_safe!(MORE_ELEMENTS).len() as f64 / (idx_ / passed as f64);

            //Format the line to be printed
            let mut line = format!(
                "{idx_} / {} {:.2}% runtime: {} ETA: {}",
                get_safe!(MORE_ELEMENTS).len(),
                precent,
                convert_time(passed as f64 / 1000f64),
                convert_time((eta - passed as f64) / 1000f64)
            );

            let line_length = line.len();

            //Make sure there is no residual chars from last line
            if line_length < last_line_length_ {
                line += &" ".repeat(last_line_length_ - line_length);
            }
            let mut last_line_length_ = last_line_length.lock().unwrap();
            *last_line_length_ = line_length;

            //Print the line and flush stdout
            //If you don;t flush stdout not every line will be printed,
            //Because print! doesn't flush as oppose to println!
            print!("\r{line}");
            std::io::stdout().flush().unwrap();
            let mut idx_ = idx.lock().unwrap();
            *idx_ += 1.0;
        }

        //
        {
            let mut elements_ = match elements.lock() {
                Ok(o) => o,
                Err(e) => {
                    println!("{:?} panic:\n{}", thread::current(), e);
                    return None;
                }
            };
            //This is intended
            #[allow(clippy::collapsible_if)]
            if e.len() < 2 {
                if !e.is_empty() {
                    if !e[0].children.is_empty() {
                        elements_.append(&mut e[0].children);
                    }
                }
                return None;
            }

            Self::append_element(&mut e, &mut elements_);
        }
        Some(data)
    }
}
