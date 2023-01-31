extern crate tokio;

mod element;
use element::*;

mod cli;
use cli::*;

mod tests;

use std::{io::Write, time::SystemTime};

async fn request(url : String) -> reqwest::Response{
    let client = reqwest::Client::new();
    match client.get(url).send().await {
        Ok(o) => o,
        Err(e) => panic!("{}", e), //TODO: add restarting
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let start = SystemTime::now();
    let cli = CLI::new(args);

    let res = request(cli.url).await;

    let data = match res.text().await {
        Ok(o) => o,
        _ => todo!(), //TODO: add restarting
    };
    println!(
        "Downloaded content in {} ms",
        start.elapsed().unwrap().as_millis()
    );
    let json_data = json::parse(&data).unwrap();

    let start = SystemTime::now();
    std::fs::write("raw.json.tmp", json_data.pretty(1)).unwrap();
    println!(
        "Written to file in {} ms",
        start.elapsed().unwrap().as_millis()
    );
    let start = std::time::SystemTime::now();

    let mut elements = Element::init(&json_data);

    unsafe {
        for more_element in &MORE_ELEMENTS {
            let url = cli.base_url.clone() + more_element + ".json";
            let res = request(url).await;

            let data = match res.text().await {
                Ok(o) => o,
                _ => todo!(), //TODO: add restarting
            };

            let json_data = json::parse(&data).unwrap();

            let mut e = Element::init(&json_data);
            ELEMENTS_COUNT -= 1;
            if e.len() < 2{
                continue;
            }
            fn app(x : &mut Vec<Element>,y : &mut Vec<Element>){
                for element in &mut *y{
                    if x.is_empty(){
                        break;
                    }
                    if *element == x[0]{
                        x.remove(0);
                        print!("Append ");
                        element.children.append(x);
                        break;
                    }
                }
                for e in y{
                    app(x,&mut e.children);
                }
            }

            app(&mut e,&mut elements);

            
            print!("Elements: {} delta: {}              \r",ELEMENTS_COUNT,start.elapsed().unwrap().as_millis());
        }
        
        if ELEMENTS_COUNT == 0 {
            panic!("Error, returned 0 elements!");
        }
    }

    let mut output: Box<dyn Write> = Box::new(std::io::stdout());

    if cli.save_to_file {
        output = Box::new(
            std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(cli.save_path)
                .unwrap(),
        );
    }

    for elem in elements {
        match output.write_fmt(format_args!("{elem:?}")) {
            Ok(()) => {}
            Err(e) => panic!("Failed to write to output!\nError: {e}"),
        }
    }
    
    unsafe {
        println!("MORE_ELEMENTS_COUNT: {MORE_ELEMENTS_COUNT}\nMORE_ELEMENTS.len(): {}\n{}",MORE_ELEMENTS.len(),MORE_ELEMENTS_COUNT == MORE_ELEMENTS.len());
        print!(
            "Successfully got {} element{} NUM_COMMENTS: {NUM_COMMENTS}",
            ELEMENTS_COUNT,
            if ELEMENTS_COUNT == 1 { "" } else { "s" }
        );
    };
    println!(", in {} ms", start.elapsed().unwrap().as_millis());
}
