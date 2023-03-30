#![cfg(test)]

#[path = "cli.rs"]
mod cli;
use cli::CLI;

use crate::{element::FORMAT, utils};

fn st(x: &str) -> String {
    x.to_string()
}

const USIZE_MAX: usize = usize::MAX;
const CLI_ELEMENT_FILTER_DEF: cli::ElementFilter = cli::ElementFilter::Default;
const CLI_ELEMENT_SORT_DEF: cli::ElementSort = cli::ElementSort::Default;

//TODO: add more
#[test]
fn test_element() {
    unsafe {
        crate::element::FORMAT = crate::element::Format::Default;
    }
    let data = include_str!("element_test_data1.json");
    let test_file_path = "test-output.tmp";

    let json_data = json::parse(data).unwrap();

    let elements = crate::element::Element::init(&json_data, USIZE_MAX);
    let mut output = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(test_file_path)
        .unwrap();
    for elem in elements {
        match std::io::Write::write_fmt(&mut output, format_args!("{elem}")) {
            Ok(()) => {}
            Err(e) => panic!("Failed to write to output!\nError: {e}"),
        }
    }
    assert_eq!(
        std::fs::read_to_string(test_file_path).unwrap(),
        include_str!("element_test_output1.txt")
            .to_owned()
            .replace('\r', "")
    );
    std::fs::remove_file(test_file_path).unwrap();
}

#[test]
fn test_cli() {
    //This test is incomplete!

    CLI::new(&[st("test-bin"), st("-h")]);
    let cli1 = CLI::new(&[st("test-bin"), st("https://reddit.com/r/asd")]);
    assert_eq!(
        (cli1.url, cli1.base_url),
        (
            st("https://reddit.com/r/asd.json"),
            st("https://reddit.com/r/asd/")
        )
    );
    let cli2 = CLI::new(&[st("test-bin"), st("https://reddit.com/r/")]);
    assert_eq!(
        (cli2.url, cli2.base_url),
        (st("https://reddit.com/r.json"), st("https://reddit.com/r/"))
    );
    assert_eq!(
        CLI::new(&[
            st("test-bin"),
            st("-s"),
            st("test-path.txt"),
            st("https://reddit.com/r/asd")
        ]),
        CLI {
            url: st("https://reddit.com/r/asd.json"),
            base_url: st("https://reddit.com/r/asd/"),
            save_to_file: true,
            save_path: st("test-path.txt"),
            max_comments: USIZE_MAX,
            filter: CLI_ELEMENT_FILTER_DEF,
            sort_style: CLI_ELEMENT_SORT_DEF,
            save_tmp_files: false,
            verbosity: cli::Verbosity::Moderate,
            req_more_elements: true,
            delete_tmp: false,
            print_timestamps: false,
        }
    );
    assert_eq!(
        CLI::new(&[
            st("test-bin"),
            st("-o"),
            st("-s"),
            st("test-path.txt"),
            st("https://reddit.com/r/")
        ]),
        CLI {
            url: st("https://reddit.com/r.json"),
            base_url: st("https://reddit.com/r/"),
            save_to_file: false,
            save_path: st("test-path.txt"),
            max_comments: USIZE_MAX,
            filter: CLI_ELEMENT_FILTER_DEF,
            sort_style: CLI_ELEMENT_SORT_DEF,
            save_tmp_files: false,
            verbosity: cli::Verbosity::Moderate,
            req_more_elements: true,
            delete_tmp: false,
            print_timestamps: false,
        }
    );
    assert_eq!(
        CLI::new(&[st("test-bin"), st("-o"), st("https://reddit.com/r/")]),
        CLI {
            url: st("https://reddit.com/r.json"),
            base_url: st("https://reddit.com/r/"),
            save_to_file: false,
            save_path: st("output.txt"),
            max_comments: USIZE_MAX,
            filter: CLI_ELEMENT_FILTER_DEF,
            sort_style: CLI_ELEMENT_SORT_DEF,
            save_tmp_files: false,
            verbosity: cli::Verbosity::Moderate,
            req_more_elements: true,
            delete_tmp: false,
            print_timestamps: false,
        }
    );

    assert_eq!(
        CLI::new(&[
            st("test-bin"),
            st("-o"),
            st("--save-tmp"),
            st("https://reddit.com/r/")
        ]),
        CLI {
            url: st("https://reddit.com/r.json"),
            base_url: st("https://reddit.com/r/"),
            save_to_file: false,
            save_path: st("output.txt"),
            max_comments: USIZE_MAX,
            filter: CLI_ELEMENT_FILTER_DEF,
            sort_style: CLI_ELEMENT_SORT_DEF,
            save_tmp_files: true,
            verbosity: cli::Verbosity::Moderate,
            req_more_elements: true,
            delete_tmp: false,
            print_timestamps: false,
        }
    );
}

#[test]
fn test_cli_parse_url() {
    assert_eq!(
        CLI::parse_url(st("https://reddit.com/r/asd")),
        (
            st("https://reddit.com/r/asd.json"),
            st("https://reddit.com/r/asd/")
        )
    );
    assert_eq!(
        CLI::parse_url(st("https://reddit.com/r/asd/?foo&bar:443")),
        (
            st("https://reddit.com/r/asd.json"),
            st("https://reddit.com/r/asd/")
        )
    );
    assert_eq!(
        CLI::parse_url(st("https://reddit.com/r/asd:443")),
        (
            st("https://reddit.com/r/asd.json"),
            st("https://reddit.com/r/asd/")
        )
    );
    assert_eq!(
        CLI::parse_url(st("https://reddit.com/r/:443")),
        (st("https://reddit.com/r.json"), st("https://reddit.com/r/"))
    );
    assert_eq!(
        CLI::parse_url(st("https://reddit.com/r/\n\n\n ")),
        (st("https://reddit.com/r.json"), st("https://reddit.com/r/"))
    );
    assert_eq!(
        CLI::parse_url(st("\nhttps://reddit.com/r/\n\n\n ")),
        (st("https://reddit.com/r.json"), st("https://reddit.com/r/"))
    );
    assert_eq!(
        CLI::parse_url(st("\nhttps://reddit.com/r//foo\n")),
        (
            st("https://reddit.com/r//foo.json"),
            st("https://reddit.com/r//foo/")
        )
    );
}

#[test]
fn test_cli_parse_sort_style() {
    //Test valid params
    assert_eq!(
        cli::CLI::parse_sort_style("default"),
        cli::ElementSort::Default
    );
    assert_eq!(cli::CLI::parse_sort_style("rand"), cli::ElementSort::Rand);
    assert_eq!(cli::CLI::parse_sort_style("random"), cli::ElementSort::Rand);
    assert_eq!(
        cli::CLI::parse_sort_style("upvotes"),
        cli::ElementSort::Upvotes(false)
    );
    assert_eq!(
        cli::CLI::parse_sort_style("ups"),
        cli::ElementSort::Upvotes(false)
    );
    assert_eq!(
        cli::CLI::parse_sort_style("upvotes-asc"),
        cli::ElementSort::Upvotes(true)
    );
    assert_eq!(
        cli::CLI::parse_sort_style("ups-asc"),
        cli::ElementSort::Upvotes(true)
    );
    assert_eq!(
        cli::CLI::parse_sort_style("comments"),
        cli::ElementSort::Comments(false)
    );
    assert_eq!(
        cli::CLI::parse_sort_style("comments-asc"),
        cli::ElementSort::Comments(true)
    );
    assert_eq!(
        cli::CLI::parse_sort_style("new"),
        cli::ElementSort::Date(false)
    );
    assert_eq!(
        cli::CLI::parse_sort_style("old"),
        cli::ElementSort::Date(true)
    );
    assert_eq!(
        cli::CLI::parse_sort_style("edited"),
        cli::ElementSort::EditedDate(false)
    );
    assert_eq!(
        cli::CLI::parse_sort_style("edited-asc"),
        cli::ElementSort::EditedDate(true)
    );

    //Test edge cases
    assert_eq!(
        cli::CLI::parse_sort_style("DefaulT"),
        cli::ElementSort::Default
    );
    assert_eq!(
        cli::CLI::parse_sort_style("Edited "),
        cli::ElementSort::EditedDate(false)
    );
    assert_eq!(
        cli::CLI::parse_sort_style("     eDiTeD"),
        cli::ElementSort::EditedDate(false)
    );
    assert_eq!(
        cli::CLI::parse_sort_style("eDiTeD-asC"),
        cli::ElementSort::EditedDate(true)
    );

    //Test invalid params
    assert_eq!(
        cli::CLI::parse_sort_style("Edi   ted   "),
        cli::ElementSort::Default
    );
    assert_eq!(cli::CLI::parse_sort_style("asd"), cli::ElementSort::Default);
    assert_eq!(
        cli::CLI::parse_sort_style("     "),
        cli::ElementSort::Default
    );
    assert_eq!(cli::CLI::parse_sort_style("\n"), cli::ElementSort::Default);
    assert_eq!(cli::CLI::parse_sort_style(""), cli::ElementSort::Default);
}

#[test]
fn test_cli_parse_format() {
    //Test valid params
    assert_eq!(
        cli::CLI::parse_format("default"),
        String::from("output.txt")
    );
    assert_eq!(get_safe!(FORMAT), crate::element::Format::Default);

    assert_eq!(cli::CLI::parse_format("d"), String::from("output.txt"));
    assert_eq!(get_safe!(FORMAT), crate::element::Format::Default);

    assert_eq!(cli::CLI::parse_format("html"), String::from("output.html"));
    assert_eq!(get_safe!(FORMAT), crate::element::Format::HTML);

    assert_eq!(cli::CLI::parse_format("h"), String::from("output.html"));
    assert_eq!(get_safe!(FORMAT), crate::element::Format::HTML);

    assert_eq!(cli::CLI::parse_format("json"), String::from("output.json"));
    assert_eq!(get_safe!(FORMAT), crate::element::Format::JSON);

    assert_eq!(cli::CLI::parse_format("j"), String::from("output.json"));
    assert_eq!(get_safe!(FORMAT), crate::element::Format::JSON);

    //Test edgecases
    assert_eq!(cli::CLI::parse_format("J"), String::from("output.json"));
    assert_eq!(get_safe!(FORMAT), crate::element::Format::JSON);

    assert_eq!(cli::CLI::parse_format("HtMl"), String::from("output.html"));
    assert_eq!(get_safe!(FORMAT), crate::element::Format::HTML);

    //Test invalid params
    assert_eq!(cli::CLI::parse_format("j"), String::from("output.json"));
    assert_eq!(get_safe!(FORMAT), crate::element::Format::JSON);

    //Reset for test_element
    unsafe {
        crate::element::FORMAT = crate::element::Format::Default;
    }
}

#[test]
fn test_cli_parse_filter_style() {
    //Test valid params

    //Edited
    assert_eq!(
        cli::CLI::parse_filter_style(&String::from("edited"), Some(&String::from("true")), None)
            .unwrap(),
        (0, cli::ElementFilter::Edited(true))
    );
    assert_eq!(
        cli::CLI::parse_filter_style(&String::from("edited"), Some(&String::from("false")), None)
            .unwrap(),
        (0, cli::ElementFilter::Edited(false))
    );

    //Author
    assert_eq!(
        cli::CLI::parse_filter_style(
            &String::from("author"),
            Some(&String::from("==")),
            Some(&String::from("test"))
        )
        .unwrap(),
        (
            0,
            cli::ElementFilter::Author(cli::ElementFilterOp::EqString(String::from("test")))
        )
    );
    assert_eq!(
        cli::CLI::parse_filter_style(
            &String::from("author"),
            Some(&String::from("!=")),
            Some(&String::from("test"))
        )
        .unwrap(),
        (
            0,
            cli::ElementFilter::Author(cli::ElementFilterOp::NotEqString(String::from("test")))
        )
    );

    //Comments
    assert_eq!(
        cli::CLI::parse_filter_style(
            &String::from("comments"),
            Some(&String::from("==")),
            Some(&String::from("123"))
        )
        .unwrap(),
        (
            2,
            cli::ElementFilter::Comments(cli::ElementFilterOp::Eq(123))
        )
    );
    assert_eq!(
        cli::CLI::parse_filter_style(
            &String::from("comments"),
            Some(&String::from("!=")),
            Some(&String::from("123"))
        )
        .unwrap(),
        (
            2,
            cli::ElementFilter::Comments(cli::ElementFilterOp::NotEq(123))
        )
    );

    assert_eq!(
        cli::CLI::parse_filter_style(
            &String::from("comments"),
            Some(&String::from(">")),
            Some(&String::from("123"))
        )
        .unwrap(),
        (
            2,
            cli::ElementFilter::Comments(cli::ElementFilterOp::Grater(123))
        )
    );

    assert_eq!(
        cli::CLI::parse_filter_style(
            &String::from("comments"),
            Some(&String::from("<")),
            Some(&String::from("123"))
        )
        .unwrap(),
        (
            2,
            cli::ElementFilter::Comments(cli::ElementFilterOp::Less(123))
        )
    );

    assert_eq!(
        cli::CLI::parse_filter_style(
            &String::from("comments"),
            Some(&String::from(">=")),
            Some(&String::from("123"))
        )
        .unwrap(),
        (
            2,
            cli::ElementFilter::Comments(cli::ElementFilterOp::GraterEq(123))
        )
    );

    assert_eq!(
        cli::CLI::parse_filter_style(
            &String::from("comments"),
            Some(&String::from("<=")),
            Some(&String::from("123"))
        )
        .unwrap(),
        (
            2,
            cli::ElementFilter::Comments(cli::ElementFilterOp::LessEq(123))
        )
    );

    //Upvotes
    assert_eq!(
        cli::CLI::parse_filter_style(
            &String::from("ups"),
            Some(&String::from("==")),
            Some(&String::from("123"))
        )
        .unwrap(),
        (
            2,
            cli::ElementFilter::Upvotes(cli::ElementFilterOp::Eq(123))
        )
    );
    assert_eq!(
        cli::CLI::parse_filter_style(
            &String::from("upvotes"),
            Some(&String::from("!=")),
            Some(&String::from("123"))
        )
        .unwrap(),
        (
            2,
            cli::ElementFilter::Upvotes(cli::ElementFilterOp::NotEq(123))
        )
    );

    assert_eq!(
        cli::CLI::parse_filter_style(
            &String::from("ups"),
            Some(&String::from(">")),
            Some(&String::from("123"))
        )
        .unwrap(),
        (
            2,
            cli::ElementFilter::Upvotes(cli::ElementFilterOp::Grater(123))
        )
    );

    assert_eq!(
        cli::CLI::parse_filter_style(
            &String::from("upvotes"),
            Some(&String::from("<")),
            Some(&String::from("123"))
        )
        .unwrap(),
        (
            2,
            cli::ElementFilter::Upvotes(cli::ElementFilterOp::Less(123))
        )
    );

    assert_eq!(
        cli::CLI::parse_filter_style(
            &String::from("ups"),
            Some(&String::from(">=")),
            Some(&String::from("123"))
        )
        .unwrap(),
        (
            2,
            cli::ElementFilter::Upvotes(cli::ElementFilterOp::GraterEq(123))
        )
    );

    assert_eq!(
        cli::CLI::parse_filter_style(
            &String::from("upvotes"),
            Some(&String::from("<=")),
            Some(&String::from("123"))
        )
        .unwrap(),
        (
            2,
            cli::ElementFilter::Upvotes(cli::ElementFilterOp::LessEq(123))
        )
    );

    //Test edge cases
    assert_eq!(
        cli::CLI::parse_filter_style(
            &String::from("UpVoteS    "),
            Some(&String::from("<=")),
            Some(&String::from("123"))
        )
        .unwrap(),
        (
            2,
            cli::ElementFilter::Upvotes(cli::ElementFilterOp::LessEq(123))
        )
    );

    assert_eq!(
        cli::CLI::parse_filter_style(
            &String::from(" \n\n\n\n\t\rEdItEd\n\n\n\n\n\t\r"),
            Some(&String::from("true")),
            None
        )
        .unwrap(),
        (0, cli::ElementFilter::Edited(true))
    );

    assert_eq!(
        cli::CLI::parse_filter_style(&String::from("EDITED"), Some(&String::from("TruE")), None)
            .unwrap(),
        (0, cli::ElementFilter::Edited(true))
    );

    assert_eq!(
        cli::CLI::parse_filter_style(&String::from("EDITED"), Some(&String::from("FAlSe")), None)
            .unwrap(),
        (0, cli::ElementFilter::Edited(false))
    );

    //Test invalid params

    assert_eq!(
        cli::CLI::parse_filter_style(&String::from("asd"), None, None).unwrap(),
        (0, cli::ElementFilter::Default)
    );

    assert_eq!(
        cli::CLI::parse_filter_style(&String::from("E dited"), None, None).unwrap(),
        (0, cli::ElementFilter::Default)
    );
}

#[test]
fn test_utils_convert_time() {
    assert_eq!(utils::convert_time(-100.0), String::from("<0.00s"));
    assert_eq!(utils::convert_time(0.0), String::from("<0.00s"));
    assert_eq!(utils::convert_time(0.99), String::from("0.99s"));
    assert_eq!(utils::convert_time(0.999), String::from("1.00s"));
    assert_eq!(utils::convert_time(10.0), String::from("10.00s"));
    assert_eq!(utils::convert_time(71.13), String::from("1min 11.13s"));
    assert_eq!(utils::convert_time(100.11), String::from("1min 40.11s"));
    assert_eq!(utils::convert_time(3671.0), String::from("1h 1min 11.00s"));
    assert_eq!(
        utils::convert_time(36710.0),
        String::from("10h 11min 50.00s")
    );
    assert_eq!(
        utils::convert_time(1000000.0),
        String::from("277h 46min 40.00s")
    );
    assert_eq!(
        utils::convert_time(1000000.789),
        String::from("277h 46min 40.79s")
    );
}
