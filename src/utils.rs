extern crate tokio;

use async_recursion::async_recursion;

use crate::{
    cli::{ElementFilter, ElementFilterOp, ElementSort},
    element::Element,
};

use rand::prelude::*;

#[async_recursion]
pub async fn request(url: String, retries: Option<usize>) -> Result<reqwest::Response,String> {
    let retries = retries.unwrap_or(0);
    let client = reqwest::Client::new();
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

//Convert time in seconds to a more readable format
// Xh Ymin Zs
pub fn convert_time(t: f64) -> String {
    if t <= 0.0 {
        String::new()
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
            .unwrap_or(Vec::new())
            .clone();
    }

    Ok(elements)
}
