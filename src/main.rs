extern crate dotenv;
extern crate tokio;
extern crate reqwest;
extern crate json;

use std::vec;

use JsonValue::Null;
use json::{object::Iter, JsonValue};
use reqwest::{Request, header};

#[derive(Debug)]
struct Empty{
 
}


#[derive(Debug,PartialEq)]
struct Element{
  data : String,
  author : String,
  children : Vec<Element>,
  
}

fn get_replies(element : &JsonValue) -> Result<Vec<String>,Empty>{
  let mut out = Vec::new();
    if element["replies"] != Null{
      // for x in element["replies"]["data"].entries(){
      //   println!("{:?}",x)
      // }
        for r in element["replies"]["data"]["children"].members(){
          //println!("{:?}",r["data"]);
         let r = &r["data"];
          //println!("{:?}",r);
          let d = [&r["author"],&r["body"]];
          let mut data = String::new();
          for &x in d.iter(){
             if x.to_owned() != JsonValue::Null{
             data += match x.as_str(){
                  Some(o)=>o,
                  _=>""
                }
              }
            }
            out.push(data);

            out.append(&mut match get_replies(r){
              Ok(o)=>o,
              _=>continue
            });

        }
    }
    if out.len() > 0 {Ok(out)} else {Err(Empty{})}
}

fn get_data(element : JsonValue,field : &str) -> Result<String,Empty>{
  if element.to_owned() != JsonValue::Null && element[field] != JsonValue::Null{
      return Ok(element[field].to_string())
  }
  Err(Empty{})
}

#[tokio::main]
async fn main(){

    let client = reqwest::Client::new();

    //let res = client.get("https://www.reddit.com/r/redditdev/comments/b8yd3r/reddit_api_possible_to_get_posts_by_id.json")
    let res = client.get("https://www.reddit.com/r/furry_irl/comments/x97xja/dead_irl.json")
    .send().await.unwrap()
    .text().await.unwrap();

  let j = json::parse(&res.clone()).unwrap();

  let mut data = Vec::<String>::new();

   for x in j.members(){
     
    for y in x["data"]["children"].members(){

      let d = y["data"].clone();
      let dc = d.clone();
      //for x in d.entries(){
       // println!("{:?}",x);
      //}
      //println!("{:?}",&d["replies"]);
      
        //TODO: Fix layout, bcs its getting all the data, now only fix the reaptionships!!!
      let d = [&d["author"],&d["body"],&d["selftext"],&d["title"]];
        
        // Element{
        //   data : match get_data(d, "body"){

        //   }
        // };

        for &x in d.iter(){
           if x.to_owned() != JsonValue::Null{
           data.push(match x.as_str(){
                Some(o)=>o.to_owned(),
                _=>"".to_owned()
              });
              println!("{}",data.pop().unwrap());
            }
          }
          match get_replies(&dc){
            Ok(o)=>println!("{:?}",o),
            _=>{}
          }
          // println!("{:?}",.unwrap());
          
    }
   }

  // println!("{:#?}",data);

    
}
