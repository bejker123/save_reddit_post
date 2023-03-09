#![allow(dead_code)]

//Allow this, bcs when running tests compiler throws a dead code warning which is not true.
#[derive(PartialEq, Eq, Debug)]
#[allow(clippy::upper_case_acronyms)] //my preference
pub struct CLI {
    pub url: String,
    pub base_url: String,
    pub save_to_file: bool,
    pub save_path: String,
    pub max_comments : usize,
    pub sort_style: ElementSort,
}

#[derive(Eq,PartialEq,Debug,Clone)]
pub enum ElementSort{
    Default,
    Rand,
    Upvotes(bool), //Ascending or not
    Comments(bool), //Ascending or not
    Date(bool), //Ascending or not
    EditedDate(bool), //Ascending or not
    
}

impl CLI {
    fn help(invalid_usage: bool) {
        println!("Usage:");
        println!("srp <arguments> <url>");
        println!("Valid arguments:");
        println!(" -h/--help display this help");
        println!(" -s/--save specify save path(output.tmp by default)");
        println!(" -o/--output don't save to file just output to stdout");
        println!(" -f/--format set the format (not case sensitive)");
        println!(" -m/--max set the max amount comments to get (min 2, to get the actual post)");
        let ll = " --sort choose sort option form:";
        let padding = " ".repeat(ll.len());
        println!("{}",ll);
        println!("{}default", padding);
        println!("{}rand", padding);
        println!("{}upvotes", padding);
        println!("{}upvotes-asc", padding);
        println!("{}comments by nr of child comments", padding);
        println!("{}comments-asc", padding);
        println!("{}new", padding);
        println!("{}old", padding);
        println!("{}edited", padding);
        println!("{}edited-asc", padding);
        let ll = " Valid formats:";
        let padding = " ".repeat(ll.len());
        println!("{}",ll);
        println!("{}Default/d", padding);
        println!("{}HTML/h", padding);

        if invalid_usage {
            println!("Invalid usage!");
        }
        if !cfg!(test) {
            std::process::exit(invalid_usage as i32);
        }
    }

    pub fn new(args: Vec<String>) -> CLI {
        let mut url = String::new();
        let mut save_to_file = true;
        let mut save_path = String::from("output.txt");
        let mut max_comments = usize::MAX;
        let mut sort_style = ElementSort::Default;

        if args.len() == 1 {
            Self::help(true);
        } else if args.len() == 2 {
            if args[1] == "-h" || args[1] == "--help" {
                Self::help(false);
            } else {
                url = args[1].clone();
            }
        } else {
            let mut skip_count = 0u32;

            for i in 1..args.len() - 1 {
                if skip_count > 0 {
                    skip_count -= 1;
                    continue;
                }
                match args[i].as_str() {
                    "-h" | "--help" => {
                        Self::help(false);
                    }
                    "-s" | "--save" => {
                        if args.len() < i + 1 {
                            Self::help(true);
                        }
                        skip_count += 1;
                        save_path = args[i + 1].clone();
                    }
                    "-o" | "--output" => {
                        save_to_file = false;
                    }
                    "-f" | "--format" => {
                        if args.len() < i + 1 {
                            Self::help(true);
                        }
                        skip_count += 1;
                        let format = args[i + 1].clone().to_lowercase();
                        match format.as_str() {
                            "default" | "d" => {
                                unsafe {
                                    crate::element::FORMAT = crate::element::ElementFormat::Default
                                }
                                save_path = String::from("output.txt")
                            }
                            "html" | "h" => {
                                unsafe {
                                    crate::element::FORMAT = crate::element::ElementFormat::HTML
                                }
                                save_path = String::from("output.html")
                            }
                            "json" | "j" => {
                                unsafe {
                                    crate::element::FORMAT = crate::element::ElementFormat::JSON
                                }
                                save_path = String::from("output.json")
                            }
                            _ => {
                                println!("Invalid format: {}", args[i + 1]);
                                Self::help(true);
                            }
                        }
                    }
                    "-m" | "--max" => {
                        if args.len() < i + 1 {
                            Self::help(true);
                        }
                        skip_count += 1;
                        let max_comments_ = args[i + 1].clone();
                        match max_comments_.parse::<usize>(){
                            Ok(o)=>max_comments = std::cmp::max(o,2),
                            Err(_)=>{
                                println!("Invalid format: {}", args[i + 1]);
                                Self::help(true);
                            }
                        }
                    }
                    "--sort" => {
                        if args.len() < i + 1 {
                            Self::help(true);
                        }
                        skip_count += 1;
                        let sort_style_ = args[i + 1].clone().trim().to_lowercase();
                        match sort_style_.as_str(){
                            "default"=>{},
                            "rand"=>sort_style = ElementSort::Rand,
                            "upvotes"=>sort_style = ElementSort::Upvotes(false),
                            "upvotes-asc"=>sort_style = ElementSort::Upvotes(true),
                            "comments"=>sort_style = ElementSort::Comments(false),
                            "comments-asc"=>sort_style = ElementSort::Comments(true),
                            "new"=>sort_style = ElementSort::Date(false),
                            "old"=>sort_style = ElementSort::Date(true),
                            "edited"=>sort_style = ElementSort::EditedDate(false), 
                            "edited-asc"=>sort_style = ElementSort::EditedDate(true), 
                            //for adding more: "tmp"=>sort_style = ElementSort::tmp, 
                            _=>{
                                println!("Invalid format: {}", args[i + 1]);
                                Self::help(true);
                            }
                        }
                    }
                    _ => {
                        println!("Invalid argument: {}", args[i])
                    }
                }
            }
            match args.last() {
                Some(o) => url = o.to_string(),
                _ => panic!("Failed to get last of args!"),
            }
            //let (url,_) = Self::parse_url(args[args.len() - 1].to_owned());
        }

        let (url, base_url) = CLI::parse_url(url);
        CLI {
            url,
            base_url,
            save_to_file,
            save_path,
            max_comments,
            sort_style
        }
    }

    pub fn parse_url(mut url: String) -> (String, String) {
        url = url.replace('\'', "");
        url = url.replace(' ', "");
        url = url.replace('\n', "");

        let search_for = '?';

        url = match url.rfind(search_for) {
            Some(idx) => url[0..idx].to_string(),
            _ => url,
        };

        let search_for = "://";

        let start_idx = match url.find(search_for) {
            Some(o) => o + search_for.len(),
            _ => 0,
        };

        url = match url[start_idx..].rfind(':') {
            Some(q_idx) => url[0..q_idx + start_idx].to_string(),
            _ => url,
        };

        let mut base_url = url.clone();

        if url.ends_with('/') {
            url = url[0..url.len() - 1].to_string();
        } else {
            base_url += "/";
        }

        if !url.ends_with(".json") {
            url += ".json";
        }

        (url, base_url)
    }
}
