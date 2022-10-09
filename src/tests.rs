#![cfg(test)]

#[path = "cli.rs"]
mod cli;
use crate::cli::*;

fn st(x: &str) -> String {
    x.to_string()
}

#[test]
fn test_cli() {
    CLI::new(vec![st("test-bin"), st("-h")]);
    assert_eq!(
        CLI::new(vec![st("test-bin"), st("https://test-url.com/")]).url,
        "https://test-url.com.json"
    );
    assert_eq!(
        CLI::new(vec![st("test-bin"), st("https://test-url.com/")]).url,
        "https://test-url.com.json"
    );
    assert_eq!(
        CLI::new(vec![
            st("test-bin"),
            st("-s"),
            st("test-path.txt"),
            st("https://test-url.com/")
        ]),
        CLI {
            url: st("https://test-url.com.json"),
            save_to_file: true,
            save_path: st("test-path.txt")
        }
    );
    assert_eq!(
        CLI::new(vec![
            st("test-bin"),
            st("-o"),
            st("-s"),
            st("test-path.txt"),
            st("https://test-url.com/")
        ]),
        CLI {
            url: st("https://test-url.com.json"),
            save_to_file: false,
            save_path: st("test-path.txt")
        }
    );
    assert_eq!(
        CLI::new(vec![st("test-bin"), st("-o"), st("https://test-url.com/")]),
        CLI {
            url: st("https://test-url.com.json"),
            save_to_file: false,
            save_path: st("output.tmp")
        }
    );
}

#[test]
fn parse_url() {
    assert_eq!(
        CLI::parse_url(st("https://test-url.com/")),
        "https://test-url.com.json"
    );
    assert_eq!(
        CLI::parse_url(st("https://test-url.com/asd/?foo&bar:443")),
        "https://test-url.com/asd.json"
    );
    assert_eq!(
        CLI::parse_url(st("https://test-url.com/asd:443")),
        "https://test-url.com/asd.json"
    );
    assert_eq!(
        CLI::parse_url(st("https://test-url.com:443")),
        "https://test-url.com.json"
    );
    assert_eq!(
        CLI::parse_url(st("https://test-url.com\n\n\n ")),
        "https://test-url.com.json"
    );
    assert_eq!(
        CLI::parse_url(st("\nhttps://test-url.com\n\n\n ")),
        "https://test-url.com.json"
    );
    assert_eq!(
        CLI::parse_url(st("\nhttps://test-url.com\n\n\n /foo")),
        "https://test-url.com/foo.json"
    );
}
