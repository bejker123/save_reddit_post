extern crate tokio;

mod element;
use element::*;

mod cli;
use cli::*;

mod tests;

use std::io::Write;

#[tokio::main]
async fn main() {
    
    let cli = CLI::new();

    let client = reqwest::Client::new();
    let res = match client.get(cli.url).send().await {
        Ok(o) => o,
        Err(e) => panic!("{}",e), //add restarting
    };

    let text = match res.text().await {
        Ok(o) => o,
        _ => todo!(), //add restarting
    };

    let j = json::parse(&text.clone()).unwrap();

    std::fs::write("raw.json.tmp", j.pretty(1)).unwrap();

    let elements = Element::init(&j);

    unsafe {
        if ELEMENTS_COUNT == 0 {
            panic!("Error, returned 0 elements!");
        }
    }

    let mut file : Box<dyn Write> = Box::new(std::io::stdout());

    if cli.save_to_file{
        file = Box::new(std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(cli.save_path)
            .unwrap());
    }

    for elem in elements {
        match file.write_fmt(format_args!("{:?}", elem)) {
            Ok(()) => {}
            Err(e) => panic!("Failed to save file!\nError: {}", e),
        }
    }
   
    unsafe {
        println!(
            "Successfully got {} element{}!",
            ELEMENTS_COUNT,
            if ELEMENTS_COUNT == 1 { "" } else { "s" }
        )
    };
}
