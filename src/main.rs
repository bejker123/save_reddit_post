extern crate tokio;

#[macro_use]
mod element;
use element::*;

mod cli;
use cli::*;

mod tests;

use std::{io::Write, time::SystemTime, process::exit};

async fn request(url : String) -> reqwest::Response{
    let client = reqwest::Client::new();
    match client.get(url).send().await {
        Ok(o) => o,
        Err(e) => panic!("{}", e), //TODO: add restarting
    }
}

//Convert time in seconds to a more readable format
// Xh Ymin Zs
fn convert_time(t : f64) -> String{
    if t == 0.0{
        String::new()
    } 
    else if t < 60.0{
        format!("{t:.2}s")
    }else if t < 3600.0{
        ((t/60.0) as i32).to_string() + "min " + &convert_time(t % 60.0)
    }else{
        ((t/3600.0) as i32).to_string() + "h " + &convert_time(t % 3600.0)
    }
}

#[tokio::main]
async fn main() {
    print!("Initialising: ");
    let args: Vec<String> = std::env::args().collect();
    let start = SystemTime::now();
    let cli = CLI::new(args);
    
    println!("success.");
    
    print!("Requesting content from {}\nStatus: ",cli.url);
    let res = request(cli.url).await;
    
    let data = match res.text().await {
        Ok(o) => {
            println!("success.");
            o
        },
        _ => {
            println!("fail.");
            exit(1);
        }, 
    };
    
    println!(
        "Downloaded content in {} ms",
        start.elapsed().unwrap().as_millis()
    );
    
    print!("Parsing to JSON: ");
    
    let json_data = match json::parse(&data){
        Ok(o)=>{
            println!("success.");
            o
        },
        _ => {
            println!("fail.");
            exit(1);
        }, 
    };
    
    
    print!("Writing to JSON file: ");
    let start = SystemTime::now();
    match std::fs::write("raw.json.tmp", json_data.pretty(1)){
        Ok(_)=>{
            print!("success ");
        },
        Err(e) => panic!("fail.\nError: {e}"), 
    }
    println!(
        "(in {} ms)",
        start.elapsed().unwrap().as_millis()
    );
    
    print!("Parsing to elements: ");
    
    let start = std::time::SystemTime::now();
    
    let mut elements = Element::init(&json_data);
    
    if !elements.is_empty(){
        println!("success.");
    }else{
        println!("fail.");
    }
    
    
    let more_start = std::time::SystemTime::now();
    
    let mut last_line_length = 0usize;
    
    //Yes I know representing index as a float is dumb.
    let mut idx = 1f64;
    
    //'more' elements
    if get_safe!(MORE_ELEMENTS_COUNT) > 0{
        println!("Getting 'more' elements:");
        
        //Get more elements from the 'more' listing
        for more_element in &get_safe!(MORE_ELEMENTS) {
            //build the url
            let url = cli.base_url.clone() + more_element + ".json";
            let res = request(url).await;
            
            let data = match res.text().await {
                Ok(o) => o,
                _ => todo!(), //TODO: add restarting
            };
            
            let json_data = json::parse(&data).unwrap();
            
            //Parse json data to elements
            let mut e = Element::init(&json_data);
            //ELEMENTS_COUNT -= 1; //TODO: remove this?
            
            //calculate % of progress as a 64bit float 64
            let precent = idx/(get_safe!(MORE_ELEMENTS).len() as f64)*100f64;
            
            //get time passed since start of getting 'more' elements 
            let passed = std::time::SystemTime::now().duration_since(more_start).unwrap().as_millis();
            
            //Get estimated time
            let eta = get_safe!(MORE_ELEMENTS).len() as f64 / (idx/ passed as f64);
            
            //Format the line to be printed
            let mut line = format!("{idx} / {} {:.2}% runtime: {} ETA: {}",
            get_safe!(MORE_ELEMENTS).len(),precent,
            convert_time(passed as f64/1000f64),
            convert_time((eta - passed as f64)/1000f64));
            
            let line_length = line.len();
            
            //Make sure there is no residual chars from last line
            if line_length < last_line_length{
                line += &" ".repeat(last_line_length-line_length);
            }
            last_line_length = line_length;
            
            //Print the line and flush stdout
            //If you don;t flush stdout not every line will be printed,
            //Because print! doesn't flush as oppose to println!
            print!("\r{line}");
            std::io::stdout().flush().unwrap();
            idx += 1.0;
            
            //
            if e.len() < 2{
                elements.append(&mut e[0].children);
                // let mut file = std::fs::OpenOptions::new()
                // .append(true)
                // .create(true)
                // .open("discarded.tmp")
                // .unwrap();
                // for x in &e{
                    //     file.write_fmt(format_args!("{x:?}\n"));
                    // }
                    continue;
                }
                
                //Recursively check every element, and if the first element matches appedn to it
                fn app(x : &mut Vec<Element>,y : &mut Vec<Element>){
                    for element in &mut *y{
                        if x.is_empty(){
                            break;
                        }
                        if *element == x[0]{
                            x.remove(0);
                            unsafe{ELEMENTS_COUNT -= 1;}
                            // print!("Append ");
                            element.children.append(x);
                            break;
                        }
                    }
                    for e in y{
                        app(x,&mut e.children);
                    }
                }
                
                app(&mut e,&mut elements);
                // print!("Elements: {} delta: {}              \r",ELEMENTS_COUNT,start.elapsed().unwrap().as_millis());
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
        print!("Writing to {}: ",cli.save_path);
    }else{
        print!("Writing to stdout: ");
    }
        
    //Write begining of the file:
    match get_safe!(FORMAT){
        ElementFormat::Default=>{
            match writeln!(output,"Source: {}",cli.base_url){
                Ok(())=>{},
                Err(e)=>panic!("fail.\nError: {e}")
            }
        },
        ElementFormat::HTML=>{
            match writeln!(output,"{}",include_str!("html_file.html").replace("{title}", &cli.base_url)){
                Ok(())=>{},
                Err(e)=>panic!("fail.\nError: {e}")
            }
        }
    }
        
        
    //Write every element to the output.
    //For formatting see element.rs:
    //                   impl std::fmt::Debug for Element
    for elem in elements {
        match write!(output,"{elem:?}") {
            //Ignore success
            Ok(()) => {},
            //If failed to write to output, panic and finish execution.
            Err(e) => panic!("fail.\nError: {e}"), 
        }
    }
        
    //Write the end:
    match get_safe!(FORMAT){
        ElementFormat::Default=>{

        },
        ElementFormat::HTML=>{
            match writeln!(output,"\t</div>\n</body>\n</html>"){
                Ok(())=>{},
                Err(e)=>panic!("fail.\nError: {e}")
            }
        }
    }
        
    println!("success");
        
    //Print last bit of debug data
    // println!("MORE_ELEMENTS_COUNT: {MORE_ELEMENTS_COUNT}\nMORE_ELEMENTS.len(): {}\n{}",MORE_ELEMENTS.len(),MORE_ELEMENTS_COUNT == MORE_ELEMENTS.len());
    //TODO: fix descrepency!!!
    print!(
        "Successfully got {} element{} NUM_COMMENTS: {}",
        get_safe!(ELEMENTS_COUNT),
        if get_safe!(ELEMENTS_COUNT) == 1 { "" } else { "s" },
        get_safe!(NUM_COMMENTS)
    );

    println!(", in {}", convert_time(start.elapsed().unwrap().as_millis() as f64 /1000f64));
}
    