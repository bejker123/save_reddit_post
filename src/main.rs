extern crate tokio;

#[macro_use]
mod element;
use element::*;

mod cli;
use cli::*;

mod output_writer;
use output_writer::*;

mod utils;
use utils::*;

mod tests;

use std::{io::Write, process::exit, sync::Arc, sync::Mutex, time::SystemTime};

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

    let elements = Element::init(&json_data, cli.max_comments);

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
    if get_safe!(MORE_ELEMENTS_COUNT) > 0 && get_safe!(ELEMENTS_COUNT) < cli.max_comments {
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
                if get_safe!(ELEMENTS_COUNT) >= cli.max_comments {
                    return;
                }
                *threads_running_.lock().unwrap() += 1;
                Element::get_more_element(
                    x.clone(),
                    idx,
                    more_start,
                    last_line_length,
                    elements,
                    cli.max_comments,
                )
                .await;
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
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        println!();
    }

    if get_safe!(ELEMENTS_COUNT) == 0 {
        panic!("Error, returned 0 elements!");
    }

    let mut elements = elements.lock().unwrap().clone();

    //Sort elements (except the first one which is the parent element or the reddit post)
    if elements.len() > 1 {
        let mut elements_cp = Vec::from([match elements.get(0) {
            Some(o) => o.clone(),
            None => panic!("Error, invalid elements!"),
        }]);
        elements_cp.append(
            &mut sort_elements(elements[1..elements.len() - 1].to_vec(), cli.sort_style)
                .unwrap_or(Vec::new()),
        );
        elements = elements_cp;
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
    for elem in elements.iter() {
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
    //TODO: fix descrepency!!!
    print!(
        "Successfully got {} element{} NUM_COMMENTS: {}",
        get_safe!(ELEMENTS_COUNT),
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
