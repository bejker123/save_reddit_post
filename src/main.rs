extern crate tokio;

#[macro_use]
mod element;
use element::*;

mod cli;
use cli::*;

mod output_writer;
use output_writer::*;

mod tests;

use async_recursion::async_recursion;
use std::{io::Write, process::exit, sync::Arc, sync::Mutex, time::SystemTime, thread};

#[async_recursion]
async fn request(url: String, retries: Option<usize>) -> reqwest::Response {
    let retries = retries.unwrap_or(0);
    let client = reqwest::Client::new();
    match client.get(url.clone()).send().await {
        Ok(o) => o,
        Err(e) => {
            if retries >= 3 {
                panic!("Max retries exeeded, error: {}", e);
            } else {
                request(url, Some(retries + 1)).await
            }
        }
    }
}

//Convert time in seconds to a more readable format
// Xh Ymin Zs
fn convert_time(t: f64) -> String {
    if t == 0.0 {
        String::new()
    } else if t < 60.0 {
        format!("{t:.2}s")
    } else if t < 3600.0 {
        ((t / 60.0) as i32).to_string() + "min " + &convert_time(t % 60.0)
    } else {
        ((t / 3600.0) as i32).to_string() + "h " + &convert_time(t % 3600.0)
    }
}

//TODO: move this to element.rs
async fn get_more_element(
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
        let precent = idx_ / (std::cmp::min(get_safe!(MORE_ELEMENTS_COUNT),max_comments) as f64) * 100f64;

        //get time passed since start of getting 'more' elements
        let passed = std::time::SystemTime::now()
            .duration_since(more_start)
            .unwrap()
            .as_millis();

        //Get estimated time
        let eta = std::cmp::min(get_safe!(MORE_ELEMENTS_COUNT),max_comments) as f64 / (idx_ / passed as f64);

        //Format the line to be printed
        let mut line = format!(
            "{idx_} / {} {:.2}% runtime: {} ETA: {}",
            std::cmp::min(get_safe!(MORE_ELEMENTS_COUNT),max_comments),
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
        if e.len() < 2 {
            if e.len() > 0{
                if e[0].children.len() > 0{
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

#[tokio::main]
async fn main() {
    print!("Initialising: ");
    let args: Vec<String> = std::env::args().collect();
    let start = SystemTime::now();
    let cli = CLI::new(args);

    println!("success.");

    print!("Requesting content from {}\nStatus: ", cli.url);
    let res = request(cli.url, None).await;

    let data = match res.text().await {
        Ok(o) => {
            println!("success.");
            o
        }
        _ => {
            println!("fail.");
            exit(1);
        }
    };

    println!(
        "Downloaded content in {} ms",
        start.elapsed().unwrap().as_millis()
    );

    print!("Parsing to JSON: ");

    let json_data = match json::parse(&data) {
        Ok(o) => {
            println!("success.");
            o
        }
        _ => {
            println!("fail.");
            exit(1);
        }
    };

    print!("Writing to JSON file: ");
    let start = SystemTime::now();
    match std::fs::write("raw.json.tmp", json_data.pretty(1)) {
        Ok(_) => {
            print!("success ");
        }
        Err(e) => panic!("fail.\nError: {e}"),
    }
    println!("(in {} ms)", start.elapsed().unwrap().as_millis());

    print!("Parsing to elements: ");

    let start = std::time::SystemTime::now();

    let elements = Element::init(&json_data,cli.max_comments);

    if !elements.is_empty() {
        println!("success.");
    } else {
        println!("fail.");
    }

    let more_start = std::time::SystemTime::now();

    let last_line_length = Arc::new(Mutex::new(0usize));

    //Yes I know representing index as a float is dumb.
    let idx = Arc::new(Mutex::new(1f64));

    let elements = Arc::new(Mutex::new(elements));

    //'more' elements
    if get_safe!(MORE_ELEMENTS_COUNT) > 0  && get_safe!(ELEMENTS_COUNT) < cli.max_comments{
        println!("Getting 'more' elements:");

        let mut threads: Vec<tokio::task::JoinHandle<_>> = Vec::new();
        let max_threads = 200;
        let threads_running = Arc::new(Mutex::new(0usize));
        //Get more elements from the 'more' listing
        for more_element in &get_safe!(MORE_ELEMENTS) {
            let x = cli.base_url.clone() + &more_element.clone() + ".json";
            let (idx, last_line_length, elements, threads_running_) = (
                Arc::clone(&idx),
                Arc::clone(&last_line_length),
                Arc::clone(&elements),
                Arc::clone(&threads_running),
            );
            let t = tokio::spawn(async move {
                if get_safe!(ELEMENTS_COUNT) >= cli.max_comments{
                    return
                }
                *threads_running_.lock().unwrap() += 1;
                get_more_element(x.clone(), idx, more_start, last_line_length, elements,cli.max_comments).await;
                *threads_running_.lock().unwrap() -= 1;
            });
            threads.push(t);
            while *threads_running.lock().unwrap() >= max_threads {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        loop {
            if *threads_running.lock().unwrap() == 0 {
                break;
            }
            //println!("{}",*threads_running.lock().unwrap());
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        println!();
    }

    if get_safe!(ELEMENTS_COUNT) == 0 {
        panic!("Error, returned 0 elements!");
    }

    //Set the default output to stdout
    let mut output: Box<dyn Write> = Box::new(std::io::stdout());

    //If user specified saving to a file.
    //Change the output to the file.
    if cli.save_to_file {
        output = Box::new(
            std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(cli.save_path.clone())
                .unwrap(),
        );
        print!("Writing to {}: ", cli.save_path);
    } else {
        print!("Writing to stdout: ");
    }
    let mut ow = OutputWriter::new();
    ow = ow.set_output(output);

    //Write begining of the file:
    match get_safe!(FORMAT) {
        ElementFormat::Default => {
            ow.content += &format!(
                "# {{indent}} {{ups}} {{author}}: {{contnet}}\n\nSource: {}",
                cli.base_url
            )
        }
        ElementFormat::HTML => {
            ow.content += &include_str!("html_file.html").replace("{title}", &cli.base_url)
        }
        ElementFormat::JSON => ow.content += "{\"data\":[",
    }

    //Write every element to the output.
    //For formatting see element.rs:
    //                   impl std::fmt::Debug for Element
    for elem in elements.lock().unwrap().iter() {
        ow.content += &format!("{elem:?}")
    }

    //Write the end:
    match get_safe!(FORMAT) {
        ElementFormat::Default => {}
        ElementFormat::HTML => ow.content += "\t</div>\n</body>\n</html>",
        ElementFormat::JSON => {
            if let Some(r) = ow.content.clone().strip_suffix(",\n") {
                ow.content = r.to_owned();
            }
            ow.content += "\n]}"
        }
    }

    match ow.write() {
        Ok(_) => println!("success"),
        Err(e) => panic!("ow.write() error:\n{}", e),
    }

    //Print last bit of debug data
    // println!("MORE_ELEMENTS_COUNT: {MORE_ELEMENTS_COUNT}\nMORE_ELEMENTS.len(): {}\n{}",MORE_ELEMENTS.len(),MORE_ELEMENTS_COUNT == MORE_ELEMENTS.len());
    //TODO: fix descrepency!!!
    print!(
        "Successfully got {} element{} NUM_COMMENTS: {}",
        if !get_safe!(MORE_ELEMENTS).is_empty() {
            get_safe!(ELEMENTS_COUNT)
        } else {
            get_safe!(ELEMENTS_COUNT)
        },
        if get_safe!(ELEMENTS_COUNT) == 1 {
            ""
        } else {
            "s"
        },
        get_safe!(NUM_COMMENTS)
    );

    println!(
        ", in {}",
        convert_time(start.elapsed().unwrap().as_millis() as f64 / 1000f64)
    );
}
