extern crate tokio;

#[macro_use]
mod element;
use cli::CLI;
use element::{
    Element, Format, ELEMENTS_COUNT, FORMAT, MORE_ELEMENTS, MORE_ELEMENTS_COUNT, NUM_COMMENTS,
};

mod cli;

mod output_writer;
use output_writer::OutputWriter;

mod utils;
use utils::{convert_time, filter_elements, request, sort_elements};

mod tests;

use std::{io::Write, sync::Arc, sync::Mutex, time::SystemTime};

fn write_to_output(
    cli: &cli::CLI,
    elements: &Vec<Element>,
    start: SystemTime,
) -> Result<(), String> {
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
            Err(e) => return Err(format!("Failed to open file with error: {e}")),
        };
        cli.print_info(format!("Writing to {}: ", cli.save_path));
    }
    let mut ow = OutputWriter::new();
    ow = ow.set_output(output);

    //Write begining of the file:
    match get_safe!(FORMAT) {
        Format::Default => {
            ow.content += &format!(
                "# {{indent}} {{ups}} {{author}}: {{contnet}}\n\nSource: {}",
                cli.base_url
            );
        }
        Format::HTML => {
            ow.content += &include_str!("html_file.html").replace("{title}", &cli.base_url);
        }
        Format::JSON => ow.content += "{\"data\":[",
    }

    //Write every element to the output.
    //For formatting see element.rs:
    //                   impl std::fmt::Debug for Element
    for elem in elements {
        ow.content += &format!("{elem}");
    }

    //Write the end:
    match get_safe!(FORMAT) {
        Format::Default => {}
        Format::HTML => ow.content += "\t</div>\n</body>\n</html>",
        Format::JSON => {
            if let Some(r) = ow.content.clone().strip_suffix(",\n") {
                ow.content = r.to_owned();
            }
            ow.content += "\n]}";
        }
    }

    match ow.write() {
        Ok(_) => {
            if cli.save_to_file {
                cli.print_info("Success");
            } else {
                cli.print_info("Writing to stdout: success");
            }
        }
        Err(e) => return Err(format!("ow.write() error:\n{e}")),
    }

    //Print last bit of debug data
    //TODO: fix descrepency!!!

    cli.print_infol(format!(
        "Successfully got {} element{}, in {}",
        get_safe!(ELEMENTS_COUNT),
        if get_safe!(ELEMENTS_COUNT) == 1 {
            ""
        } else {
            "s"
        },
        convert_time(start.elapsed().unwrap().as_secs_f64())
    ));

    let diff = get_safe!(NUM_COMMENTS) - get_safe!(ELEMENTS_COUNT);
    if diff != 0 {
        cli.print_infom(format!(
            "Not all elements've been gotten, difference: {diff}"
        ));
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let (cli, json_data) = utils::init().await;

    let start = std::time::SystemTime::now();

    let elements = Element::init(&json_data, cli.max_comments);

    if elements.is_empty() {
        cli.print_err("Parsing to elements: fail.");
    } else {
        cli.print_info("Parsing to elements: success.");
    }

    let more_start = std::time::SystemTime::now();

    let last_line_length = Arc::new(Mutex::new(0usize));

    //Yes I know representing index as a float is dumb.
    let idx = Arc::new(Mutex::new(1f64));

    let elements = Arc::new(Mutex::new(elements));

    let more_elems_dir_path = &(utils::TMP_DIR.to_owned() + "more_elements");
    let more_elements_dir = std::path::Path::new(more_elems_dir_path);
    if cli.save_tmp_files && !more_elements_dir.exists() {
        std::fs::create_dir(more_elements_dir).unwrap();
    }

    //'more' elements
    if get_safe!(MORE_ELEMENTS_COUNT) > 0 && get_safe!(ELEMENTS_COUNT) < cli.max_comments {
        if cli.req_more_elements {
            cli.print_infom("Getting 'more' elements:");

            let max_threads = std::thread::available_parallelism().unwrap().get();
            cli.print_info(format!("Running {max_threads} threads"));
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
                tokio::spawn(async move {
                    if get_safe!(ELEMENTS_COUNT) >= cli.max_comments {
                        return;
                    }
                    *threads_running_.lock().unwrap() += 1;
                    if let Some(o) = Element::get_more_element(
                        &cli.verbosity,
                        cli.print_timestamps,
                        x.clone(),
                        idx,
                        more_start,
                        last_line_length,
                        elements,
                        cli.max_comments,
                    )
                    .await
                    {
                        if cli.save_tmp_files {
                            let filename = x.replace(&base_url, "").trim().to_owned();
                            let mut file =
                                std::fs::File::create(more_elements_dir + "/" + &filename).unwrap();
                            file.write_fmt(format_args!("{o}")).unwrap();
                        }
                    }
                    *threads_running_.lock().unwrap() -= 1;
                });
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
        } else {
            cli.print_infom("Not requesting more elements, because of --no-more-elements flag");
        }
    }

    assert!(
        get_safe!(ELEMENTS_COUNT) != 0,
        "Error, returned 0 elements!"
    );

    let mut elements = elements
        .lock()
        .map_or_else(|_| cli.print_err("Failed to lock elements!"), |e| e.clone());

    //Sort elements (except the first one which is the parent element or the reddit post)
    if elements.len() > 1 {
        //Filter elements.
        elements = match filter_elements(elements, cli.filter.clone(), vec![]) {
            Some(o) => o.0,
            None => cli.print_err("Error, no elements, after filtering."),
        };
        //Sort elements.
        if elements.len() > 2 {
            let mut elements_cp = Vec::from([elements.get(0).map_or_else(
                || cli.print_err("Error, invalid elements!"),
                std::clone::Clone::clone,
            )]);
            elements_cp.append(
                &mut sort_elements(elements[1..elements.len() - 1].to_vec(), cli.sort_style)
                    .unwrap_or_default(),
            );
            elements = elements_cp;
        }
    }

    if let Err(e) = write_to_output(&cli, &elements, start) {
        cli.print_err(format!("Writing to output failed: {e}"));
    }
    if cli.delete_tmp {
        if let Err(e) = utils::delete_tmp() {
            cli.print_warning(e);
        }
    }
}
