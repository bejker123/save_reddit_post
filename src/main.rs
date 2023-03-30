extern crate tokio;

#[macro_use]
mod element;

use element::{
    Element, ELEMENTS_COUNT, MORE_ELEMENTS, MORE_ELEMENTS_COUNT,
};

mod cli;
mod output_writer;

mod utils;
use utils::{convert_time, request};

mod tests;

use std::{io::Write, sync::Arc, sync::Mutex};

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
    elements = utils::sort_elements_(elements,&cli);

    if let Err(e) = utils::write_to_output(&cli, &elements, start) {
        cli.print_err(format!("Writing to output failed: {e}"));
    }
    if cli.delete_tmp {
        if let Err(e) = utils::delete_tmp() {
            cli.print_warning(e);
        }
    }
}
