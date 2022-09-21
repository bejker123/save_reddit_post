use std::io::Write;

pub struct CLI{
    pub url : String,
}

impl CLI{
    pub fn new() -> CLI{
        let mut url = String::new();

        let args: Vec<String> = std::env::args().collect();

        if args.len() >= 2 {
            url = args[1].clone();
        } else {
            print!("URL: ");
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(&mut url).unwrap();
        }

        let url = CLI::parse_url(url);
        CLI{url}
    }

    fn parse_url(mut url: String) -> String {
        url = url.replace("\"", "");
        url = url.replace(" ", "");
        url = url.replace("\n", "");
        
        let search_for = "?";
    
        url = match url.rfind(search_for) {
            Some(idx) => url[0..idx].to_string(),
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
}
