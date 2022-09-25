#[cfg(test)]

#[path ="cli.rs"]
mod cli;
use crate::cli::*;

#[test]
fn parse_url(){
    assert_eq!(CLI::parse_url("https://test-url.com/".to_string()),"https://test-url.com.json");
    assert_eq!(CLI::parse_url("https://test-url.com/asd/?foo&bar:443".to_string()),"https://test-url.com/asd.json");
    assert_eq!(CLI::parse_url("https://test-url.com/asd:443".to_string()),"https://test-url.com/asd.json");
    assert_eq!(CLI::parse_url("https://test-url.com:443".to_string()),"https://test-url.com.json");
    assert_eq!(CLI::parse_url("https://test-url.com\n\n\n ".to_string()),"https://test-url.com.json");
    assert_eq!(CLI::parse_url("\nhttps://test-url.com\n\n\n ".to_string()),"https://test-url.com.json");
    assert_eq!(CLI::parse_url("\nhttps://test-url.com\n\n\n /foo".to_string()),"https://test-url.com/foo.json");
}