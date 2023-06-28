extern crate tokio;

use async_recursion::async_recursion;
use console::{style, StyledObject};
use json::JsonValue;
use std::{
    io::Write,
    time::{self, SystemTime},
};

use crate::{
    cli::{self, ElementFilter, ElementFilterOp, ElementSort, Verbosity, CLI},
    element::{Element, Format, ELEMENTS_COUNT, FORMAT, NUM_COMMENTS},
    output_writer::OutputWriter,
};

use rand::prelude::*;

pub const TMP_DIR: &str = "tmp/";

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/114.0";

#[async_recursion]
pub async fn request(url: String, retries: Option<usize>) -> Result<reqwest::Response, String> {
    let retries = retries.unwrap_or(0);
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .or(Err(String::from("Failed to build client.")))?;
    match client.get(url.clone()).send().await {
        Ok(o) => Ok(o),
        Err(e) => {
            if retries >= 3 {
                Err(format!("Max retries exeeded, error: {e}"))
            } else {
                request(url, Some(retries + 1)).await
            }
        }
    }
}

pub fn get_timestamp(get: bool) -> StyledObject<String> {
    if get {
        let now = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        style(format!(
            "[{}.{}] ",
            chrono::Local::now().format("%H:%M:%S"),
            format!("{:.3}", now - now.floor()).replace("0.", "")
        ))
        .yellow()
        .bold()
    } else {
        style(String::new())
    }
}

//Convert time in seconds to a more readable format
// Xh Ymin Zs
pub fn convert_time(t: f64) -> String {
    if t <= 0.0 {
        String::from("<0.00s")
    } else if t < 60.0 {
        format!("{t:.2}s")
    } else if t < 3600.0 {
        format!("{:.0}min {}", ((t / 60.0).floor()), convert_time(t % 60.0))
    } else {
        format!(
            "{:.0}h {}",
            ((t / 3600.0).floor()),
            convert_time(t % 3600.0)
        )
    }
}

/*New design:
 * Start from the bottom, and go up.
 * Filter out any elements that don't meet either condition:
 *  a) Matching the filter parameters
 *  b) A child matches the filter parameters.
 *
 * That shuld result only in the elements that match the filter and their parents.
 */
pub fn filter_elements(
    mut elements: Vec<Element>,
    filter: ElementFilter,
    mut req_elements: Vec<String>, //Required elements' id
) -> Option<(Vec<Element>, Vec<String>)> {
    if elements.is_empty() {
        return None;
    }
    for element in &mut elements {
        //filter children recursively

        if let Some(o) = filter_elements(
            element.children.clone(),
            filter.clone(),
            req_elements.clone(),
        ) {
            let (children, mut req) = o;
            element.children = children;
            for r in req.clone() {
                if !req.contains(&r) && !req_elements.contains(&r) {
                    req.push(r);
                }
            }
            req_elements = req;
        }
    }

    match filter {
        ElementFilter::Default => return Some((elements, req_elements)),
        ElementFilter::Upvotes(o) => {
            match o {
                //Reamember to reverse number operators:
                ElementFilterOp::Eq(o) => {
                    elements.retain(|a| a.ups == o || req_elements.contains(&a.id));
                }
                ElementFilterOp::NotEq(o) => {
                    elements.retain(|a| a.ups != o || req_elements.contains(&a.id));
                }
                ElementFilterOp::Less(o) => {
                    elements.retain(|a| a.ups < o || req_elements.contains(&a.id));
                }
                ElementFilterOp::LessEq(o) => {
                    elements.retain(|a| a.ups <= o || req_elements.contains(&a.id));
                }
                ElementFilterOp::Grater(o) => {
                    elements.retain(|a| a.ups > o || req_elements.contains(&a.id));
                }
                ElementFilterOp::GraterEq(o) => {
                    elements.retain(|a| a.ups >= o || req_elements.contains(&a.id));
                }
                _ => {} //invalid for this type(ElementFilter::Upvotes), only number operators apply
            }
        }
        ElementFilter::Author(o) => {
            match o {
                ElementFilterOp::EqString(o) => {
                    elements.retain(|a| a.author == o || req_elements.contains(&a.id));
                }
                ElementFilterOp::NotEqString(o) => {
                    elements.retain(|a| a.author != o || req_elements.contains(&a.id));
                }
                _ => {} //invalid for this type(ElementFilter::Author), only string operations apply
            }
        }
        ElementFilter::Edited(o) => {
            elements.retain(|a| a.edited.eq(&usize::MAX) != o);
        }
        ElementFilter::Comments(o) => {
            match o {
                //Reamember to reverse number operators:
                ElementFilterOp::Eq(o) => {
                    elements.retain(|a| a.children.len() == o || req_elements.contains(&a.id));
                }
                ElementFilterOp::NotEq(o) => {
                    elements.retain(|a| a.children.len() != o || req_elements.contains(&a.id));
                }
                ElementFilterOp::Less(o) => {
                    elements.retain(|a| a.children.len() > o || req_elements.contains(&a.id));
                }
                ElementFilterOp::LessEq(o) => {
                    elements.retain(|a| a.children.len() >= o || req_elements.contains(&a.id));
                }
                ElementFilterOp::Grater(o) => {
                    elements.retain(|a| a.children.len() < o || req_elements.contains(&a.id));
                }
                ElementFilterOp::GraterEq(o) => {
                    elements.retain(|a| a.children.len() <= o || req_elements.contains(&a.id));
                }
                _ => {} //invalid for this type(ElementFilter::Comments), only number operators apply
            }
        }
    }

    for element in elements.clone() {
        req_elements.push(element.parent_id);
    }

    Some((elements, req_elements))
}

pub fn write_to_output(
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
        cli.print_info_nn(format!("Writing to {}: ", cli.save_path));
    }
    let mut ow = OutputWriter::new();
    ow.set_output(output);

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
            if cli.verbosity == Verbosity::High {
                if cli.save_to_file {
                    println!("Success");
                } else {
                    println!("Writing to stdout: success");
                }
            }
        }
        Err(e) => return Err(format!("Failed to write to output with error:\n{e}")),
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

    let diff = (get_safe!(NUM_COMMENTS) as i32 - get_safe!(ELEMENTS_COUNT) as i32).abs();
    if diff != 0 {
        cli.print_info(format!("Difference: {diff}"));
    }
    Ok(())
}

pub fn sort_elements_(mut elements: Vec<Element>, cli: &CLI) -> Vec<Element> {
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
    elements
}

pub fn sort_elements(
    mut elements: Vec<Element>,
    sort_style: ElementSort,
) -> Result<Vec<Element>, String> {
    if elements.is_empty() {
        return Err(String::from("elements empty"));
    }

    match sort_style {
        ElementSort::Default => return Ok(elements),
        ElementSort::Rand => {
            let mut rng = rand::thread_rng();
            elements.shuffle(&mut rng);
        }
        ElementSort::Upvotes(false) => elements.sort_by(|a, b| b.ups.cmp(&a.ups)),
        ElementSort::Upvotes(true) => elements.sort_by(|a, b| a.ups.cmp(&b.ups)),
        ElementSort::Comments(false) => {
            elements.sort_by(|a, b| b.children.len().cmp(&a.children.len()));
        }
        ElementSort::Comments(true) => {
            elements.sort_by(|a, b| a.children.len().cmp(&b.children.len()));
        }
        ElementSort::Date(false) => elements.sort_by(|a, b| b.created.cmp(&a.created)),
        ElementSort::Date(true) => elements.sort_by(|a, b| a.created.cmp(&b.created)),
        ElementSort::EditedDate(false) => elements.sort_by(|a, b| b.edited.cmp(&a.edited)),
        ElementSort::EditedDate(true) => elements.sort_by(|a, b| a.edited.cmp(&b.edited)),
    }

    for element in &mut elements {
        //sort children recursively
        element.children = sort_elements(element.children.clone(), sort_style)
            .unwrap_or_default()
            .clone();
    }

    Ok(elements)
}

pub fn delete_tmp() -> Result<(), String> {
    std::fs::remove_dir_all(TMP_DIR).map_err(|_| String::from("Failed to delete temp files!"))?;
    Ok(())
}

pub async fn init() -> (CLI, JsonValue) {
    let args: Vec<String> = std::env::args().collect();
    let start = time::SystemTime::now();
    let cli = crate::cli::CLI::new(&args);

    cli.print_info("Initialising CLI: success");
    cli.print_infom(format!("Requesting content from {}:", cli.url));
    let Ok(res) = request(cli.url.clone(), None).await else{
        CLI::print_err_no_timestamp("Fail");
    };

    let data = (res.text().await).map_or_else(
        |_| {
            CLI::print_err_no_timestamp("Fail");
        },
        |o| {
            cli.print_infom(format!(
                "Success in {}",
                convert_time(start.elapsed().unwrap().as_secs_f64())
            ));
            o
        },
    );

    let json_data = json::parse(&data).map_or_else(
        |e| {
            println!("{data}");
            CLI::print_err_no_timestamp(format!("Parsing to JSON error: {e}"));
        },
        |o| {
            cli.print_info("Parsing to JSON: success");
            o
        },
    );

    if cli.save_tmp_files {
        let tmp_dir = std::path::Path::new(TMP_DIR);
        if !tmp_dir.exists() {
            std::fs::create_dir(tmp_dir).unwrap();
        }

        match std::fs::write(TMP_DIR.to_owned() + "raw.json", json_data.pretty(1)) {
            Ok(_) => {
                cli.print_info("Writing to JSON file: success");
            }
            Err(e) => {
                CLI::print_err_no_timestamp(format!("Writing to JSON file: fail.\nError: {e}"))
            }
        }
    }

    if cli.save_tmp_files {
        let tmp_dir = std::path::Path::new(TMP_DIR);
        if !tmp_dir.exists() {
            std::fs::create_dir(tmp_dir).unwrap();
        }

        match std::fs::write(TMP_DIR.to_owned() + "raw.json", json_data.pretty(1)) {
            Ok(_) => {
                cli.print_info("Writing to JSON file: success");
            }
            Err(e) => {
                CLI::print_err_no_timestamp(format!("Writing to JSON file: fail.\nError: {e}"))
            }
        }
    }
    cli.print_info(format!(
        "Initialising success, in {}",
        convert_time(start.elapsed().unwrap().as_secs_f64())
    ));

    (cli, json_data)
}
