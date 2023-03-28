use console::style;

//Allow this, bcs when running tests compiler throws a dead code warning which is not true.
#[derive(PartialEq, Eq, Debug, Clone)]
#[allow(clippy::upper_case_acronyms)] //my preference
pub struct CLI {
    pub url: String,
    pub base_url: String,
    pub save_to_file: bool,
    pub save_path: String,
    pub max_comments: usize,
    pub sort_style: ElementSort,
    pub filter: ElementFilter,
    pub save_tmp_files: bool,
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum ElementSort {
    Default,
    Rand,
    Upvotes(bool),    //Ascending or not
    Comments(bool),   //Ascending or not
    Date(bool),       //Ascending or not
    EditedDate(bool), //Ascending or not
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ElementFilterOp {
    //ElementFilterOperator
    EqString(String),
    NotEqString(String),

    Eq(usize),
    NotEq(usize),
    Grater(usize),
    GraterEq(usize),
    Less(usize),
    LessEq(usize),
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ElementFilter {
    Default, //Don't filter
    Upvotes(ElementFilterOp),
    Comments(ElementFilterOp),
    Author(ElementFilterOp),
    Edited(bool),
}

impl CLI {
    fn print_arg(arg: &str, desc: &str) {
        println!(" {} {}", style(arg).yellow().bold(), style(desc).blue());
    }

    fn help(invalid_usage: bool) {
        println!("{}", style("Usage:").green().bold());
        println!(
            "{} {} {}",
            style("srp").green(),
            style("<arguments>").yellow().bold(),
            style("<url>").magenta().bold()
        );
        println!("{}", style("Valid arguments:").green().bold());

        Self::print_arg("-h/--help", "display this help");
        Self::print_arg("-s/--save", "specify save path(output.tmp by default)");
        Self::print_arg("-o/--output", "don't save to file just output to stdout");
        Self::print_arg(
            "-m/--max",
            "set the max amount comments to get (min 2, to get the actual post)",
        );
        Self::print_arg(
            "--save-tmp-files",
            "allow saving temp files (raw json data)",
        );
        Self::print_arg("-f/--format", "set the format (not case sensitive)");

        let padding = '\t';
        println!(" {}", style("Valid formats:").green().bold());
        println!("{padding}{}", style("Default/d").yellow());
        println!("{padding}{}", style("HTML/h").yellow());
        println!("{padding}{}", style("JSON/j").yellow());

        Self::print_arg("--sort", "choose sort option form:");
        println!("{padding}{}", style("default").yellow());
        println!("{padding}{}", style("rand/random").yellow());
        println!("{padding}{}", style("upvotes/ups").yellow());
        println!("{padding}{}", style("upvotes/ups-asc").yellow());
        println!(
            "{padding}{}",
            style("comments by nr of child comments").yellow()
        );
        println!("{padding}{}", style("comments-asc").yellow());
        println!("{padding}{}", style("new").yellow());
        println!("{padding}{}", style("old").yellow());
        println!("{padding}{}", style("edited").yellow());
        println!("{padding}{}", style("edited-asc").yellow());

        Self::print_arg(
            "--filter",
            "add filter option from: (multiple allowed as separate args)",
        );
        println!(
            "{padding}{}",
            style("[data] [operator] [value]").yellow().bold()
        );
        println!(
            "{padding}{}",
            style("ups/upvotes > >= == < <= != [nr]").yellow()
        );
        println!(
            "{padding}{}",
            style("comments > >= == < <= != [nr]").yellow()
        );
        println!(
            "{padding}{}",
            style("edited [bool]").yellow()
        );
        println!(
            "{padding}{}",
            style("author == != [value]").yellow()
        );

        if invalid_usage {
            println!("{}", style("Invalid usage!").bold().red());
        }
        if !cfg!(test) {
            std::process::exit(0);
        }
    }

    pub fn parse_format(format: &str) -> String {
        let mut save_path = String::from("output.txt");

        match format.to_lowercase().trim() {
            "default" | "d" => {
                unsafe {
                    crate::element::FORMAT = crate::element::ElementFormat::Default;
                }
                save_path = String::from("output.txt");
            }
            "html" | "h" => {
                unsafe {
                    crate::element::FORMAT = crate::element::ElementFormat::HTML;
                }
                save_path = String::from("output.html");
            }
            "json" | "j" => {
                unsafe {
                    crate::element::FORMAT = crate::element::ElementFormat::JSON;
                }
                save_path = String::from("output.json");
            }
            _ => {
                println!("Invalid format: {format}");
                Self::help(true);
            }
        }
        save_path
    }

    pub fn parse_sort_style(sort_style_: &str) -> ElementSort {
        match sort_style_.to_lowercase().trim() {
            "default" => ElementSort::Default,
            "rand" | "random" => ElementSort::Rand,
            "upvotes" | "ups" => ElementSort::Upvotes(false),
            "upvotes-asc" | "ups-asc" => ElementSort::Upvotes(true),
            "comments" => ElementSort::Comments(false),
            "comments-asc" => ElementSort::Comments(true),
            "new" => ElementSort::Date(false),
            "old" => ElementSort::Date(true),
            "edited" => ElementSort::EditedDate(false),
            "edited-asc" => ElementSort::EditedDate(true),
            //for adding more: "tmp"=>ElementSort::tmp,
            _ => {
                println!("Invalid sort option: {sort_style_}");
                Self::help(true);
                ElementSort::Default
            }
        }
    }

    pub fn parse_filter_style(
        filter_: &String,
        operator: Option<&String>,
        value: Option<&String>,
    ) -> (u32, ElementFilter) {
        let mut filter = ElementFilter::Default;
        let mut skip_count = 0;
        match filter_.to_lowercase().trim() {
            "ups" | "upvotes" => {
                let value = value.unwrap();
                skip_count += 2;
                match operator.unwrap().as_str() {
                    ">" => match value.parse::<usize>() {
                        Ok(o) => {
                            filter = ElementFilter::Upvotes(ElementFilterOp::Grater(o));
                        }
                        Err(_) => {
                            println!("Invalid argument in filter: {filter_}");
                        }
                    },
                    ">=" => match value.parse::<usize>() {
                        Ok(o) => {
                            filter = ElementFilter::Upvotes(ElementFilterOp::GraterEq(o));
                        }
                        Err(_) => {
                            println!("Invalid argument in filter: {filter_}");
                        }
                    },
                    "==" => match value.parse::<usize>() {
                        Ok(o) => {
                            filter = ElementFilter::Upvotes(ElementFilterOp::Eq(o));
                        }
                        Err(_) => {
                            println!("Invalid argument in filter: {filter_}");
                        }
                    },
                    "!=" => match value.parse::<usize>() {
                        Ok(o) => {
                            filter = ElementFilter::Upvotes(ElementFilterOp::NotEq(o));
                        }
                        Err(_) => {
                            println!("Invalid argument in filter: {filter_}");
                        }
                    },
                    "<" => match value.parse::<usize>() {
                        Ok(o) => {
                            filter = ElementFilter::Upvotes(ElementFilterOp::Less(o));
                        }
                        Err(_) => {
                            println!("Invalid argument in filter: {filter_}");
                        }
                    },
                    "<=" => match value.parse::<usize>() {
                        Ok(o) => {
                            filter = ElementFilter::Upvotes(ElementFilterOp::LessEq(o));
                        }
                        Err(_) => {
                            println!("Invalid argument in filter: {filter_}");
                        }
                    },
                    _ => println!("Invalid argument in filter: {filter_}"),
                }
            }
            "comments" => {
                let value = value.unwrap();
                skip_count += 2;
                match operator.unwrap().as_str() {
                    ">" => match value.parse::<usize>() {
                        Ok(o) => {
                            filter = ElementFilter::Comments(ElementFilterOp::Grater(o));
                        }
                        Err(_) => {
                            println!("Invalid argument in filter: {filter_}");
                        }
                    },
                    ">=" => match value.parse::<usize>() {
                        Ok(o) => {
                            filter = ElementFilter::Comments(ElementFilterOp::GraterEq(o));
                        }
                        Err(_) => {
                            println!("Invalid argument in filter: {filter_}");
                        }
                    },
                    "==" => match value.parse::<usize>() {
                        Ok(o) => {
                            filter = ElementFilter::Comments(ElementFilterOp::Eq(o));
                        }
                        Err(_) => {
                            println!("Invalid argument in filter: {filter_}");
                        }
                    },
                    "!=" => match value.parse::<usize>() {
                        Ok(o) => {
                            filter = ElementFilter::Comments(ElementFilterOp::NotEq(o));
                        }
                        Err(_) => {
                            println!("Invalid argument in filter: {filter_}");
                        }
                    },
                    "<" => match value.parse::<usize>() {
                        Ok(o) => {
                            filter = ElementFilter::Comments(ElementFilterOp::Less(o));
                        }
                        Err(_) => {
                            println!("Invalid argument in filter: {filter_}");
                        }
                    },
                    "<=" => match value.parse::<usize>() {
                        Ok(o) => {
                            filter = ElementFilter::Comments(ElementFilterOp::LessEq(o));
                        }
                        Err(_) => {
                            println!("Invalid argument in filter: {filter_}");
                        }
                    },
                    _ => println!("Invalid argument in filter: {filter_}"),
                }
            }
            "edited" => {
                let operator = operator.unwrap().to_lowercase();
                if operator.trim() == "false" {
                    filter = ElementFilter::Edited(false);
                } else {
                    filter = ElementFilter::Edited(true);
                }
            }
            "author" => {
                let operator = operator.unwrap().to_lowercase();
                if operator.trim() == "==" {
                    filter =
                        ElementFilter::Author(ElementFilterOp::EqString(value.unwrap().clone()));
                } else if operator.trim() == "!=" {
                    filter =
                        ElementFilter::Author(ElementFilterOp::NotEqString(value.unwrap().clone()));
                } else {
                    println!("Invalid operator in filter: {operator}");
                }
            }
            _ => println!("Invalid argument in filter: {filter_}"),
        };
        (skip_count, filter)
    }

    pub fn new(args: &[String]) -> CLI {
        let mut url = String::new();
        let mut save_to_file = true;
        let mut save_path = String::from("output.txt");
        let mut max_comments = usize::MAX;
        let mut sort_style = ElementSort::Default;
        let mut filter = ElementFilter::Default; //ElementFilter::Comments(ElementFilterOp::Grater(1));//ElementFilter::Edited(false);//ElementFilter::Author(ElementFilterOp::NotEqString(String::from("funambula")));
        let mut save_tmp_files = false;

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
                        save_path = Self::parse_format(format.as_str());
                    }
                    "-m" | "--max" => {
                        if args.len() < i + 1 {
                            Self::help(true);
                        }
                        skip_count += 1;
                        let max_comments_ = args[i + 1].clone();
                        if let Ok(o) = max_comments_.parse::<usize>() {
                            max_comments = std::cmp::max(o, 2);
                        } else {
                            println!("Invalid format: {}", args[i + 1]);
                            Self::help(true);
                        }
                    }
                    "--sort" => {
                        if args.len() < i + 1 {
                            Self::help(true);
                        }
                        skip_count += 1;
                        let sort_style_ = args[i + 1].clone().trim().to_lowercase();
                        sort_style = Self::parse_sort_style(sort_style_.as_str());
                    }
                    "--filter" => {
                        if args.len() < i + 1 {
                            Self::help(true);
                        }
                        skip_count += 1;
                        let (skip_count_inc, filter_) = Self::parse_filter_style(
                            args.get(i + 1).unwrap(),
                            args.get(i + 2),
                            args.get(i + 3),
                        );
                        filter = filter_;
                        skip_count += skip_count_inc;
                    }
                    "--save-tmp-files" => {
                        save_tmp_files = true;
                    }
                    _ => {
                        println!("Invalid argument: {}", args[i]);
                    }
                }
            }
            match args.last() {
                Some(o) => url = o.to_string(),
                _ => panic!("Failed to get last of args!"),
            }
        }

        let (url, base_url) = CLI::parse_url(url);
        CLI {
            url,
            base_url,
            save_to_file,
            save_path,
            max_comments,
            sort_style,
            filter,
            save_tmp_files,
        }
    }

    pub fn parse_url(mut url: String) -> (String, String) {
        if !url.contains("reddit.com/") {
            println!("Invalid url: {url}");
            Self::help(true);
        }

        url = url.replace('\'', "");
        url = url.replace(' ', "");
        url = url.replace('\n', "");

        let search_for = '?';

        url = match url.rfind(search_for) {
            Some(idx) => url[0..idx].to_string(),
            _ => url,
        };

        let search_for = "https://";

        let start_idx = if let Some(o) = url.find(search_for) {
            o + search_for.len()
        } else {
            url = search_for.to_owned() + &url;
            search_for.len() - 1
        };

        url = match url[start_idx..].rfind(':') {
            Some(q_idx) => url[0..q_idx + start_idx].to_string(),
            _ => url,
        };

        let mut base_url = url.clone();

        //If url doens't contain at least 3 / it's assumed to be invalid
        if url.matches('/').count() < 3 {
            println!("Invalid url: {url}");
            Self::help(true);
        }

        if url.ends_with('/') {
            url = url[0..url.len() - 1].to_string();
        } else {
            base_url += "/";
        }

        if !std::path::Path::new(&url)
            .extension()
            .map_or(false, |ext| ext.eq_ignore_ascii_case("json"))
        {
            url += ".json";
        }

        (url, base_url)
    }
}
