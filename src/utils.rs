extern crate tokio;
use async_recursion::async_recursion;

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