#![cfg(test)]

#[path = "cli.rs"]
mod cli;

use crate::cli::*;

fn st(x: &str) -> String {
    x.to_string()
}

//TODO: add more
#[test]
fn test_element(){

    let data = include_str!("element_test_data1.json");
    let test_file_path = "test-output.tmp";

    let json_data = json::parse(&data).unwrap();

    let elements = crate::element::Element::init(&json_data);
    let mut output = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(test_file_path)
            .unwrap();
    for elem in elements {
        match std::io::Write::write_fmt(&mut output, format_args!("{elem:?}")) {
            Ok(()) => {}
            Err(e) => panic!("Failed to write to output!\nError: {e}"),
        }
    }
    assert_eq!(std::fs::read_to_string(test_file_path).unwrap(),include_str!("element_test_output1.txt").to_owned().replace("\r", ""))
}


#[test]
fn test_cli() {
    CLI::new(vec![st("test-bin"), st("-h")]);
    let cli1 = CLI::new(vec![st("test-bin"), st("https://test-url.com/")]);
    assert_eq!(
        (cli1.url,cli1.base_url),
        (st("https://test-url.com.json"),st("https://test-url.com/"))
    );
    let cli2 =  CLI::new(vec![st("test-bin"), st("https://test-url.com/")]);
    assert_eq!(
        (cli2.url,cli2.base_url),
        (st("https://test-url.com.json"),st("https://test-url.com/"))
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
            base_url: st("https://test-url.com/"),
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
            base_url: st("https://test-url.com/"),
            save_to_file: false,
            save_path: st("test-path.txt")
        }
    );
    assert_eq!(
        CLI::new(vec![st("test-bin"), st("-o"), st("https://test-url.com/")]),
        CLI {
            url: st("https://test-url.com.json"),
            base_url: st("https://test-url.com/"),
            save_to_file: false,
            save_path: st("output.txt")
        }
    );
}

#[test]
fn parse_url() {
    assert_eq!(
        CLI::parse_url(st("https://test-url.com/")),
        (st("https://test-url.com.json"),st("https://test-url.com/"))
    );
    assert_eq!(
        CLI::parse_url(st("https://test-url.com/asd/?foo&bar:443")),
        (st("https://test-url.com/asd.json"),st("https://test-url.com/asd/"))
    );
    assert_eq!(
        CLI::parse_url(st("https://test-url.com/asd:443")),
        (st("https://test-url.com/asd.json"),st("https://test-url.com/asd/"))
    );
    assert_eq!(
        CLI::parse_url(st("https://test-url.com:443")),
        (st("https://test-url.com.json"),st("https://test-url.com/"))
    );
    assert_eq!(
        CLI::parse_url(st("https://test-url.com\n\n\n ")),
        (st("https://test-url.com.json"),st("https://test-url.com/"))
    );
    assert_eq!(
        CLI::parse_url(st("\nhttps://test-url.com\n\n\n ")),
        (st("https://test-url.com.json"),st("https://test-url.com/"))
    );
    assert_eq!(
        CLI::parse_url(st("\nhttps://test-url.com\n\n\n /foo")),
        (st("https://test-url.com/foo.json"),st("https://test-url.com/foo/"))
    );
}
