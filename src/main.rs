extern crate tokio;

#[macro_use]
mod element;
use element::*;

mod cli;
use cli::CLI;

mod output_writer;
use output_writer::OutputWriter;

mod utils;
use utils::{convert_time, filter_elements, request, sort_elements};

mod tests;

use std::{io::Write, process::exit, sync::Arc, sync::Mutex, time::SystemTime};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let start = SystemTime::now();
    let cli = CLI::new(&args);

    println!("Initialising: success.");

    print!("Requesting content from {}\nStatus: ", cli.url);
    let res = request(cli.url, None).await;

    let data = if let Ok(o) = res.text().await {
        println!("success.");
        o
    } else {
        println!("fail.");
        exit(1);
    };

    println!(
        "Downloaded content in {} ms",
        start.elapsed().unwrap().as_millis()
    );

    print!("Parsing to JSON: ");

    let json_data = if let Ok(o) = json::parse(&data) {
        println!("success.");
        o
    } else {
        println!("fail.");
        exit(1);
    };

    let start = SystemTime::now();

    if cli.save_tmp_files{
        let tmp_dir = std::path::Path::new("tmp");
        if !tmp_dir.exists(){
            std::fs::create_dir(tmp_dir).unwrap();
        }
        print!("Writing to JSON file: ");
        match std::fs::write("tmp/raw.json", json_data.pretty(1)) {
            Ok(_) => {
                print!("success ");
            }
            Err(e) => panic!("fail.\nError: {e}"),
        }
    }
    println!("(in {} ms)", start.elapsed().unwrap().as_millis());

    print!("Parsing to elements: ");

    let start = std::time::SystemTime::now();

    let elements = Element::init(&json_data, cli.max_comments);

    if elements.is_empty() {
        println!("fail.");
    } else {
        println!("success.");
    }

    let more_start = std::time::SystemTime::now();

    let last_line_length = Arc::new(Mutex::new(0usize));

    //Yes I know representing index as a float is dumb.
    let idx = Arc::new(Mutex::new(1f64));

    let elements = Arc::new(Mutex::new(elements));

    let more_elements_dir = std::path::Path::new("tmp/more_elements");
    if cli.save_tmp_files{
        if !more_elements_dir.exists(){
            std::fs::create_dir(more_elements_dir).unwrap();
        }
    }

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
            let base_url = cli.base_url.clone();
            let more_elements_dir = more_elements_dir.to_str().unwrap().to_owned();
            let t = tokio::spawn(async move {
                if get_safe!(ELEMENTS_COUNT) >= cli.max_comments {
                    return;
                }
                *threads_running_.lock().unwrap() += 1;
                if let Some(o) = Element::get_more_element(
                    x.clone(),
                    idx,
                    more_start,
                    last_line_length,
                    elements,
                    cli.max_comments,
                )
                .await{
                    if cli.save_tmp_files{
                        let filename = x.replace(&base_url, "").trim().to_owned();
                        let mut file = std::fs::File::create(more_elements_dir+"/"+&filename).unwrap();
                        file.write_fmt(format_args!("{o}")).unwrap();
                    }
                }
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

    assert!(
        get_safe!(ELEMENTS_COUNT) != 0,
        "Error, returned 0 elements!"
    );

    let mut elements = elements.lock().unwrap().clone();

    //Sort elements (except the first one which is the parent element or the reddit post)
    if elements.len() > 1 {
        //Filter elements.
        elements = match filter_elements(elements, cli.filter, vec![]) {
            Some(o) => o.0,
            None => panic!("Error, no elements, after filtering."),
        };
        //Sort elements.
        if elements.len() > 2 {
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
    }

    //Set the default output to stdout
    let mut output: Box<dyn Write> = Box::new(std::io::stdout());

    //If user specified saving to a file.
    //Change the output to the file.
    if cli.save_to_file {
        output = match std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(cli.save_path.clone())
        {
            Ok(o) => Box::new(o),
            Err(e) => panic!("Failed to open file with error: {e}"),
        };
        print!("Writing to {}: ", cli.save_path);
    }
    let mut ow = OutputWriter::new();
    ow = ow.set_output(output);

    //Write begining of the file:
    match get_safe!(FORMAT) {
        ElementFormat::Default => {
            ow.content += &format!(
                "# {{indent}} {{ups}} {{author}}: {{contnet}}\n\nSource: {}",
                cli.base_url
            );
        }
        ElementFormat::HTML => {
            ow.content += &include_str!("html_file.html").replace("{title}", &cli.base_url);
        }
        ElementFormat::JSON => ow.content += "{\"data\":[",
    }

    //Write every element to the output.
    //For formatting see element.rs:
    //                   impl std::fmt::Debug for Element
    for elem in &elements {
        ow.content += &format!("{elem:?}");
    }

    //Write the end:
    match get_safe!(FORMAT) {
        ElementFormat::Default => {}
        ElementFormat::HTML => ow.content += "\t</div>\n</body>\n</html>",
        ElementFormat::JSON => {
            if let Some(r) = ow.content.clone().strip_suffix(",\n") {
                ow.content = r.to_owned();
            }
            ow.content += "\n]}";
        }
    }

    match ow.write() {
        Ok(_) => {
            if !cli.save_to_file {
                print!("Writing to stdout: ");
            }
            println!("success");
        }
        Err(e) => panic!("ow.write() error:\n{e}"),
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
