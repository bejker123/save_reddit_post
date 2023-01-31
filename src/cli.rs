#![allow(dead_code)]
//Allow this, bcs when running tests compiler throws a dead code warning which is not true.
#[derive(PartialEq, Eq, Debug)]
pub struct CLI {
    pub url: String,
    pub base_url: String,
    pub save_to_file: bool,
    pub save_path: String,
}

impl CLI {
    fn help(invalid_usage: bool) {
        println!("Usage:");
        println!("srp <arguments> <url>");
        println!("Valid arguments:");
        println!(" -h/--help display this help");
        println!(" -s/--save specify save path(output.tmp by default)");
        println!(" -o/--output don't save to file just output to stdout");

        if invalid_usage{
            println!("Invalid usage!");
        }
        if !cfg!(test){
            std::process::exit(invalid_usage as i32);
        }
    }

    pub fn new(args: Vec<String>) -> CLI {
        let mut url = String::new();
        let mut save_to_file = true;
        let mut save_path = String::from("output.tmp");

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
                    _ => {
                        println!("Invalid argument: {}", args[i])
                    }
                }
            }
            match args.last(){
                Some(o)=> url = o.to_string(),
                _=>panic!("Failed to get last of args!")
            } 
            //let (url,_) = Self::parse_url(args[args.len() - 1].to_owned());
        }

        let (url,base_url) = CLI::parse_url(url);
        CLI {
            url,
            base_url,
            save_to_file,
            save_path,
        }
    }

    pub fn parse_url(mut url: String) -> (String,String) {
        print!("{url}");
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
        print!(" {url}");

        url = match url[start_idx..].rfind(':') {
            Some(q_idx) => url[0..q_idx + start_idx].to_string(),
            _ => url,
        };

        let mut base_url = url.clone();

        if url.ends_with('/') {
            url = url[0..url.len() - 1].to_string();
        }else{
            base_url += "/";
        }

        if !url.ends_with(".json") {
            url += ".json";
        }
        println!(" {url}");

        (url,base_url)
    }
}
