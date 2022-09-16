extern crate json;
extern crate reqwest;
extern crate tokio;

use std::io::Write;
use std::str::FromStr;

use json::JsonValue;
use JsonValue::Null;

#[derive(Debug)]
struct Empty;

//TODO: add better debug formatting
#[derive(PartialEq)]
struct Element {
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
        // f.debug_struct("Element")
        // .field("author", &self.author)
        // .field("data", &self.data)
        // .field("kind", &self.kind)
        // .field("post_hint", &self.post_hint)
        // .field("url", &self.url)
        // .field("ups", &self.ups)
        // .field("created_utc", &self.created_utc)
        // .field("depth", &self.depth)

        // .field("children", &self.children)
        // .finish()

        let mut children = String::new();

        for child in &self.children {
            children += &format!("{:?}", child);
        }

        // let children = format!("{:?}",self.children).replace("[", "\n").replace("]", "").replace(",", "");
        if self.data != "" || self.author != "" {
            let indent_char = "-";
            //let secondary_indent_char = " ";
            let indent = indent_char.repeat(usize::from_str(&self.depth).unwrap_or(0));
            return f.write_fmt(format_args!(
                "{}{} {}: {}\n{}",
                indent,
                self.depth,
                self.author,
                self.data.replace(
                    "\n",
                    &(String::from("\n")
                        + &(indent.to_string() + &indent_char.repeat(self.author.len() + 4)))
                            //.replace(indent_char, secondary_indent_char))
                ),
                children
            ));
        }
        return Ok(());
    }
}

// impl std::fmt::Debug for Element{
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        // f.debug_struct("Element").field("author", &self.author).field("data", &self.data).field("kind", &self.kind).field("post_hint", &self.post_hint).field("url", &self.url).field("ups", &self.ups).field("children", &self.children).finish()
//         f.
//         // for x in list_members!(self){
//         // }
//     //     f.write_str(&format!("{{\"author\": \"{}\",",self.author)).unwrap();
//     //     if self.data.len() > 0{
//     //        f.write_str(&format!("\"data\": \"{}\",",self.data)).unwrap();
//     //     }
//     //     if self.kind.len() > 0{
//     //       f.write_str(&format!("\"kind\": \"{}\",",self.kind)).unwrap();
//     //    }
//     //    if self.post_hint.len() > 0{
//     //       f.write_str(&format!("\"post_hint\": \"{}\",",self.post_hint)).unwrap();
//     //    }
//     //    if self.url.len() > 0{
//     //     f.write_str(&format!("\"url\": \"{}\",",self.url)).unwrap();
//     //     }
//     //     if self.ups != 0{
//     //       f.write_str(&format!("\"ups\": \"{}\",",self.ups)).unwrap();
//     //    }
//     //    if self.children.len() > 0{
//     //     f.write_str(&format!("\"children\": \"{:#?}\",",self.children)).unwrap();
//     //  }
//     //   f.write_fmt(format_args!("{{\"author\": \"{}\",\"data\": \"{}\",\"kind\": \"{}\",\"post_hint\": \"{}\","));
//        //f.write_str(stringify!(self))
//       }
// }

impl Element {
    pub fn create(data: &JsonValue) -> Result<Element, Empty> {
        if data.to_owned() == JsonValue::Null {
            return Err(Empty {});
        }
        unsafe {
            ELEMENTS_COUNT += 1;
        } 
        Ok(Element {
            author: match get_data(data, "author") {
                Ok(o) => o,
                _ => String::new(),
            },
            data: match get_data(data, "body") {
                Ok(o) => o,
                _ => match get_data(data, "selftext") {
                    Ok(o) => o,
                    _ => match get_data(data, "title") {
                        //TODO: fix error, when a post has text the title doesn't get saved!
                        //debug this, but this is most likeley fixed.
                        Ok(o) => o,
                        _ => String::new(),
                    },
                },
            },
            children: match get_replies(data) {
                Ok(o) => o,
                _ => Vec::new(),
            },
            ups: match get_data(data, "ups") {
                Ok(o) => match o.parse::<usize>() {
                    Ok(o) => o,
                    _ => 0,
                },
                _ => 0,
            },
            post_hint: match get_data(data, "post_hint") {
                Ok(o) => o,
                _ => String::new(),
            },
            url: match get_data(data, "url_overridden_by_dest") {
                Ok(o) => o,
                _ => String::new(),
            },
            //a hacky way, but "kind" attribute is higher in the json tree so it would be a pain in the butt to get it that way
            kind: match get_data(data, "name") {
                Ok(o) => o[0..2].to_owned(),
                _ => String::new(),
            },
            created_utc: match get_data(data, "created_utc") {
                Ok(o) => o,
                _ => String::new(),
            },
            depth: match get_data(data, "depth") {
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
    if out.len() > 0 {
        Ok(out)
    } else {
        Err(Empty {})
    }
}

static mut ELEMENTS_COUNT: u128 = 0;

fn get_data(element: &JsonValue, field: &str) -> Result<String, Empty> {
    if element[field] != JsonValue::Null {
        return Ok(element[field].to_string());
    }
    Err(Empty {})
}

fn parse_url(mut url: String) -> String {
    url = match url.strip_suffix("\n") {
        Some(o) => o.to_string(),
        _ => url,
    };

    url = match url.strip_suffix("?") {
        Some(o) => o.to_string(),
        _ => url,
    };

    let search_for = "://";

    let start_idx = match url.find(search_for) {
        Some(o) => o + search_for.len(),
        _ => 0,
    };

    url = match url[start_idx..].rfind(":") {
        Some(q_idx) => url[0..q_idx + start_idx].to_string(),
        _ => url,
    };

    if url.ends_with("/") {
        url = url[0..url.len() - 1].to_string();
    }

    if !url.ends_with(".json") {
        url += ".json";
    }

    url
}

#[tokio::main]
async fn main() {
    // let x = Element::empty();

    let mut url = String::new();

    let args: Vec<String> = std::env::args().collect();

    if args.len() >= 2 {
        url = args[1].clone();
    } else {
        print!("URL: ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut url).unwrap();
    }

    let url = parse_url(url);

    //TODO: add better handling than unwrap()
    let client = reqwest::Client::new();
    let res = client.get(url).send().await.unwrap().text().await.unwrap();

    let j = json::parse(&res.clone()).unwrap();

    std::fs::write("tmp.json.tmp", j.pretty(1)).unwrap();

    let mut elements = Vec::<Element>::new();

    for x in j.members() {
        for y in x["data"]["children"].members() {
            let data = y["data"].clone();

            match Element::create(&data) {
                Ok(o) => elements.push(o),
                _ => elements.push(Element::empty()),
            }
        }
    }

    //std::fs::write("temp.tmp",format!("{:?}",elements)).unwrap();

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        //.append(true)
        .open("temp.tmp")
        .unwrap();

    for elem in elements {
        file.write_fmt(format_args!("{:?}", elem));
    }

    unsafe {
        println!(
            "Successfully got {} element{}!",
            ELEMENTS_COUNT,
            if ELEMENTS_COUNT == 1 { "" } else { "s" }
        )
    };
}
