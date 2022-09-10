extern crate dotenv;
extern crate tokio;
extern crate reqwest;
extern crate json;

use std::io::Write;

use JsonValue::Null;
use json::{JsonValue};
#[derive(Debug)]
struct Empty{
 
}


#[derive(Debug,PartialEq)]
struct Element{
  author : String,
  data : String,
  children : Vec<Element>,
  ups : usize,
}

impl Element{
  pub fn create(data : &JsonValue) -> Result<Element,Empty>{
    if data.to_owned() == JsonValue::Null{
      return Err(Empty{})
    }
    Ok(Element{
      author : match get_data(data,"author"){
        Ok(o)=>o,
        _=>String::new()
      },
      data : match get_data(data, "body"){
          Ok(o)=>o,
          _=> match get_data(data,"selftext"){
              Ok(o)=>o,
              _=>match get_data(data,"title"){//TODO: fix error, when a post has text the title doesn't get saved!
                Ok(o)=>o,
                _=>String::new()
              }
            }
      },
      children : match get_replies(data){
        Ok(o)=>o,
        _=>Vec::new()
      },
      ups : match get_data(data,"ups"){
        Ok(o)=>match o.parse::<usize>(){
          Ok(o)=>o,
          _=>0
        },
        _=>0
      }
    })
  }

  pub fn empty() -> Element{
    Element { author:String::new(), data: String::new(), children: Vec::<Element>::new(), ups: 0 }
  }
}

fn get_replies(element : &JsonValue) -> Result<Vec<Element>,Empty>{
  let mut out = Vec::<Element>::new();
    if element["replies"] != Null{

        for r in element["replies"]["data"]["children"].members(){
          let element = match Element::create(&r["data"]){
              Ok(o)=>o,
              _=>continue
          };

          out.push(element);

        }
    }
    if out.len() > 0 {Ok(out)} else {Err(Empty{})}
}

fn get_data(element : &JsonValue,field : &str) -> Result<String,Empty>{
  if element[field] != JsonValue::Null{
      return Ok(element[field].to_string())
  }
  Err(Empty{})
}


#[tokio::main]
async fn main(){

  let mut url = String::new();

  let args: Vec<String> = std::env::args().collect(); 

  if args.len() >= 2{
      url = args[1].clone();
  }
  else{
    print!("URL: ");
    std::io::stdout().flush();
    std::io::stdin().read_line(&mut url);
  }

  if url.ends_with("/"){
    url = url[0..url.len()-1].to_string();
  }

  url += ".json";

    let client = reqwest::Client::new();
//TODO: add better handling than unwrap()
    //let res = client.get("https://www.reddit.com/r/redditdev/comments/b8yd3r/reddit_api_possible_to_get_posts_by_id.json")
    let res = client.get(url)
    .send().await.unwrap()
    .text().await.unwrap();

  let j = json::parse(&res.clone()).unwrap();

  let mut elements = Vec::<Element>::new();

  for x in j.members(){
     
    for y in x["data"]["children"].members(){

      let data = y["data"].clone();

      //let dc = d.clone();
      //for x in d.entries(){
       // println!("{:?}",x);
      //}
      //println!("{:?}",&d["replies"]);
      
        //TODO: Fix layout, bcs its getting all the data, now only fix the reaptionships!!!
        match Element::create(&data){
            Ok(o)=>elements.push(o),
            _=>elements.push(Element::empty())
        }

          // match get_replies(&dc){
          //   Ok(o)=>println!("{:?}",o),
          //   _=>{}
          // }
          // println!("{:?}",.unwrap());
          
    }
   }

   std::fs::write("essa",format!("{:#?}",elements));

  // println!("{:#?}",data);

    
}
