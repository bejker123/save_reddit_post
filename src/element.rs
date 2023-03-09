extern crate json;
extern crate reqwest;

use std::{str::FromStr, sync::{Arc, Mutex}, time::SystemTime, thread};

use json::JsonValue;

use crate::{request, convert_time};
use std::io::Write;

#[derive(Debug)]
pub struct Empty;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy)]
pub enum ElementFormat {
    Default,
    HTML,
    JSON,
}

pub static mut NUM_COMMENTS: usize = 0;
pub static mut ELEMENTS_COUNT: usize = 1;
pub static mut MORE_ELEMENTS_COUNT: usize = 0;
pub static mut MORE_ELEMENTS: Vec<String> = Vec::new();
pub static mut FORMAT: ElementFormat = ElementFormat::Default;

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
    author: String,
    data: String,
    kind: String,
    url: String, //url_overridden_by_dest
    pub ups: usize,
    pub children: Vec<Element>,
    depth: String,
    permalink: String,
    id: String,
    over_18: bool,
    pub created : usize,
    pub edited : usize,
}

impl PartialEq for Element {
    fn eq(&self, other: &Element) -> bool {
        self.id == other.id
    }
}

impl std::fmt::Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.data.is_empty() || self.author.is_empty() {
            return std::fmt::Result::Err(std::fmt::Error::default());
        }
        return match get_safe!(FORMAT) {
            ElementFormat::Default => {
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
            ElementFormat::HTML => {
                let children = self
                    .children
                    .iter()
                    .map(|x| format!("{x:?}"))
                    .collect::<String>();
                let indent_char = " ";
                let indent = "\t".to_owned()
                    + &indent_char.repeat(usize::from_str(&self.depth).unwrap_or(0));
                let url = if !self.url.is_empty() {
                    format!("<a href=\"{}\">{}</a>", self.url, self.url)
                } else {
                    String::new()
                };
                f.write_fmt(format_args!(
                    "\n{indent}<div class=\"element\">\n\t
                    {indent}<h4><a href=\"{}\">{}</a> ⬆️{}:</h4>
                    {url}
                    <span>{}</span>
                    <ul>{children}</ul>
                    \n{indent}
                    </div>", //TODO: add human readable formatting
                    String::from("https://reddit.com") + &self.permalink,
                    self.author,
                    self.ups,
                    self.data.strip_prefix(&self.url).unwrap(),
                ))
            }
            ElementFormat::JSON => {
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
    pub fn create(child: &JsonValue, max_elements: usize ) -> Result<Element, Empty> {
        if get_safe!(ELEMENTS_COUNT) >= max_elements{
            return Err(Empty {})
        }
        let data = &child["data"];
        if *data == JsonValue::Null {
            return Err(Empty {})
        }
        //If the element lists more elements(it's kind is more)
        if child["kind"].clone() == "more" {
            unsafe {
                MORE_ELEMENTS_COUNT += match data["count"].as_usize() {
                    Some(o) => o,
                    _ => 0,
                }
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
                NUM_COMMENTS = match n_comments.parse::<usize>() {
                    Ok(o) => o,
                    _ => 0,
                }
            }
        }

        let _title = get_data_wrapper!(data, title, String::new());
        let url = get_data_wrapper!(data, url, String::new());
        let selftext = get_data_wrapper!(data, selftext, String::new());
        let body = get_data_wrapper!(data, body, String::new());
        add_to_total(url);
        add_to_total(_title);
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
            Element {
                author: author,
                data: total_data,
                children: match Element::get_replies(data,max_elements) {
                    Ok(o) => o,
                    _ => Vec::new(),
                },
                ups: match get_data_wrapper!(data, ups, "0".to_string()).parse::<usize>() {
                    Ok(o) => o,
                    _ => 0usize,
                },
                //post_hint: get_data_wrapper!(data, "post_hint", String::new()),
                url: get_data_wrapper!(data, url_overridden_by_dest, String::new()),
                //a hacky way, but "kind" attribute is higher in the json tree so it would be a pain in the butt to get it that way
                kind: get_data_wrapper!(data, name, String::new())[0..2].to_owned(),
                //edited: get_data_wrapper!(data, edited, String::from("false")) == *"true",
                depth: get_data_wrapper!(data, depth, "0".to_string()),
                permalink: get_data_wrapper!(data, permalink, String::new()),
                id: get_data_wrapper!(data, id, String::new()),
                over_18: get_data_wrapper!(data, over_18, String::from("false")) == *"true",
                created: match get_data_wrapper!(data, created, usize::MAX.to_string()).parse::<f32>() {
                    Ok(o) => o as usize,
                    _ => usize::MAX,
                },
                edited: match get_data_wrapper!(data, edited, usize::MAX.to_string()).parse::<f32>() {
                    Ok(o) => o as usize,
                    _ => usize::MAX,
                },
            },
        )
    }

    pub fn init(data: &JsonValue,max_elements: usize) -> Vec<Element> {
        let mut elements = Vec::<Element>::new();

        for member in data.members() {
            for child in member["data"]["children"].members() {
                if get_safe!(ELEMENTS_COUNT) >= max_elements{
                    break
                }
                //.If created element isn't empty (Ok) push it.
                if let Ok(o) = Element::create(child,max_elements) {
                    elements.push(o)
                }
            }
        }
        elements
    }

    fn get_replies(element: &JsonValue,max_elements: usize) -> Result<Vec<Element>, Empty> {
        let mut out = Vec::<Element>::new();
        if element["replies"] != JsonValue::Null {
            for child in element["replies"]["data"]["children"].members() {
                if get_safe!(ELEMENTS_COUNT) >= max_elements{
                    break
                }
                let element = match Element::create(child,max_elements) {
                    Ok(o) => o,
                    _ => continue,
                };

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

    pub async fn get_more_element(
        base_url: String,
        idx: Arc<Mutex<f64>>,
        more_start: SystemTime,
        last_line_length: Arc<Mutex<usize>>,
        elements: Arc<Mutex<Vec<Element>>>,
        max_comments : usize
    ) {
        if get_safe!(ELEMENTS_COUNT) >= max_comments{
            return
        }
        //build the url
        let url = base_url;
        let res = request(url, None).await;
    
        let data = match res.text().await {
            Ok(o) => o,
            _ => todo!(), //TODO: add restarting
        };
    
        let json_data = json::parse(&data).unwrap();
    
        //Parse json data to elements
        let mut e = Element::init(&json_data,max_comments);
    
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
            let mut elements_ = match elements.lock(){
                Ok(o)=>o,
                Err(e)=>{
                    println!("{:?} panic:\n{}",thread::current(),e);
                    return;
                }
            };
            //This is intended
            #[allow(clippy::collapsible_if)]
            if e.len() < 2 {
                if !e.is_empty(){
                    if !e[0].children.is_empty(){
                        elements_.append(&mut e[0].children);
                    }
                }
                return;
            }
    
            //Recursively check every element, and if the first element matches appedn to it
            fn app(x: &mut Vec<Element>, y: &mut Vec<Element>) {
                for element in &mut *y {
                    if x.is_empty() {
                        break;
                    }
                    if *element == x[0] {
                        x.remove(0);
                        unsafe {
                            ELEMENTS_COUNT -= 1;
                        }
                        // print!("Append ");
                        element.children.append(x);
                        break;
                    }
                }
                for e in y {
                    app(x, &mut e.children);
                }
            }
    
            app(&mut e, &mut elements_);
        }
        // print!("Elements: {} delta: {}              \r",ELEMENTS_COUNT,start.elapsed().unwrap().as_millis());
        //println!();
    }
    

}
