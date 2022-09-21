extern crate tokio;

mod element;
use element::*;

mod cli;
use cli::*;

use std::io::Write;

#[tokio::main]
async fn main() {
    
    let cli = CLI::new();

    //TODO: add better handling than unwrap()
    let client = reqwest::Client::new();
    let res = match client.get(cli.url).send().await {
        Ok(o) => o,
        _ => todo!(), //add restarting
    };

    let text = match res.text().await {
        Ok(o) => o,
        _ => todo!(), //add restarting
    };

    let j = json::parse(&text.clone()).unwrap();

    std::fs::write("tmp.json.tmp", j.pretty(1)).unwrap();

    let elements = Element::init(&j);

    unsafe {
        if ELEMENTS_COUNT == 0 {
            panic!("Error, returned 0 elements!");
        }
    }

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("temp.tmp")
        .unwrap();

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
