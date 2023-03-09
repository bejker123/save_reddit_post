extern crate tokio;
use async_recursion::async_recursion;

use crate::{cli::ElementSort, element::Element};

use rand::prelude::*;

#[async_recursion]
pub async fn request(url: String, retries: Option<usize>) -> reqwest::Response {
    let retries = retries.unwrap_or(0);
    let client = reqwest::Client::new();
    match client.get(url.clone()).send().await {
        Ok(o) => o,
        Err(e) => {
            if retries >= 3 {
                panic!("Max retries exeeded, error: {}", e);
            } else {
                request(url, Some(retries + 1)).await
            }
        }
    }
}

//Convert time in seconds to a more readable format
// Xh Ymin Zs
pub fn convert_time(t: f64) -> String {
    if t == 0.0 {
        String::new()
    } else if t < 60.0 {
        format!("{t:.2}s")
    } else if t < 3600.0 {
        ((t / 60.0) as i32).to_string() + "min " + &convert_time(t % 60.0)
    } else {
        ((t / 3600.0) as i32).to_string() + "h " + &convert_time(t % 3600.0)
    }
}

pub fn sort_elements(
    mut elements: Vec<Element>,
    sort_style: ElementSort,
) -> Result<Vec<Element>, String> {
    if elements.is_empty() {
        return Err(String::from("elements empty"));
    }

    match sort_style {
        ElementSort::Rand => {
            let mut rng = rand::thread_rng();
            elements.shuffle(&mut rng);
        }
        ElementSort::Upvotes(false) => elements.sort_by(|a, b| b.ups.cmp(&a.ups)),
        ElementSort::Upvotes(true) => elements.sort_by(|a, b| a.ups.cmp(&b.ups)),
        ElementSort::Comments(false) => {
            elements.sort_by(|a, b| b.children.len().cmp(&a.children.len()))
        }
        ElementSort::Comments(true) => {
            elements.sort_by(|a, b| a.children.len().cmp(&b.children.len()))
        }
        ElementSort::Date(false) => elements.sort_by(|a, b| b.created.cmp(&a.created)),
        ElementSort::Date(true) => elements.sort_by(|a, b| a.created.cmp(&b.created)),
        ElementSort::EditedDate(false) => elements.sort_by(|a, b| b.edited.cmp(&a.edited)),
        ElementSort::EditedDate(true) => elements.sort_by(|a, b| a.edited.cmp(&b.edited)),
        ElementSort::Default => {}
    }

    for element in &mut elements {
        //sort children recursively
        element.children = sort_elements(element.children.clone(), sort_style.clone())
            .unwrap_or(Vec::new())
            .to_vec();
    }

    Ok(elements)
}
