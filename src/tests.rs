#![cfg(test)]

#[path = "cli.rs"]
mod cli;
use cli::CLI;

use crate::element::FORMAT;

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
        crate::element::FORMAT = crate::element::ElementFormat::Default;
    }
    let data = include_str!("element_test_data1.json");
    let test_file_path = "test-output.tmp";

    let json_data = json::parse(&data).unwrap();

    let elements = crate::element::Element::init(&json_data, USIZE_MAX);
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
    assert_eq!(
        std::fs::read_to_string(test_file_path).unwrap(),
        include_str!("element_test_output1.txt")
            .to_owned()
            .replace("\r", "")
    )
}

#[test]
fn test_cli() {
    //This test is incomplete!

    CLI::new(&vec![st("test-bin"), st("-h")]);
    let cli1 = CLI::new(&vec![st("test-bin"), st("https://test-url.com/")]);
    assert_eq!(
        (cli1.url, cli1.base_url),
        (st("https://test-url.com.json"), st("https://test-url.com/"))
    );
    let cli2 = CLI::new(&vec![st("test-bin"), st("https://test-url.com/")]);
    assert_eq!(
        (cli2.url, cli2.base_url),
        (st("https://test-url.com.json"), st("https://test-url.com/"))
    );
    assert_eq!(
        CLI::new(&vec![
            st("test-bin"),
            st("-s"),
            st("test-path.txt"),
            st("https://test-url.com/")
        ]),
        CLI {
            url: st("https://test-url.com.json"),
            base_url: st("https://test-url.com/"),
            save_to_file: true,
            save_path: st("test-path.txt"),
            max_comments: USIZE_MAX,
            filter: CLI_ELEMENT_FILTER_DEF,
            sort_style: CLI_ELEMENT_SORT_DEF,
            save_tmp_files: false,
        }
    );
    assert_eq!(
        CLI::new(&vec![
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
            save_path: st("test-path.txt"),
            max_comments: USIZE_MAX,
            filter: CLI_ELEMENT_FILTER_DEF,
            sort_style: CLI_ELEMENT_SORT_DEF,
            save_tmp_files: false,
        }
    );
    assert_eq!(
        CLI::new(&vec![st("test-bin"), st("-o"), st("https://test-url.com/")]),
        CLI {
            url: st("https://test-url.com.json"),
            base_url: st("https://test-url.com/"),
            save_to_file: false,
            save_path: st("output.txt"),
            max_comments: USIZE_MAX,
            filter: CLI_ELEMENT_FILTER_DEF,
            sort_style: CLI_ELEMENT_SORT_DEF,
            save_tmp_files: false,
        }
    );

    assert_eq!(
        CLI::new(&vec![st("test-bin"), st("-o"), st("--save-tmp-files"), st("https://test-url.com/")]),
        CLI {
            url: st("https://test-url.com.json"),
            base_url: st("https://test-url.com/"),
            save_to_file: false,
            save_path: st("output.txt"),
            max_comments: USIZE_MAX,
            filter: CLI_ELEMENT_FILTER_DEF,
            sort_style: CLI_ELEMENT_SORT_DEF,
            save_tmp_files: true,
        }
    );
}

#[test]
fn test_cli_parse_url() {
    assert_eq!(
        CLI::parse_url(st("https://test-url.com/")),
        (st("https://test-url.com.json"), st("https://test-url.com/"))
    );
    assert_eq!(
        CLI::parse_url(st("https://test-url.com/asd/?foo&bar:443")),
        (
            st("https://test-url.com/asd.json"),
            st("https://test-url.com/asd/")
        )
    );
    assert_eq!(
        CLI::parse_url(st("https://test-url.com/asd:443")),
        (
            st("https://test-url.com/asd.json"),
            st("https://test-url.com/asd/")
        )
    );
    assert_eq!(
        CLI::parse_url(st("https://test-url.com:443")),
        (st("https://test-url.com.json"), st("https://test-url.com/"))
    );
    assert_eq!(
        CLI::parse_url(st("https://test-url.com\n\n\n ")),
        (st("https://test-url.com.json"), st("https://test-url.com/"))
    );
    assert_eq!(
        CLI::parse_url(st("\nhttps://test-url.com\n\n\n ")),
        (st("https://test-url.com.json"), st("https://test-url.com/"))
    );
    assert_eq!(
        CLI::parse_url(st("\nhttps://test-url.com\n\n\n /foo")),
        (
            st("https://test-url.com/foo.json"),
            st("https://test-url.com/foo/")
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
    assert_eq!(get_safe!(FORMAT), crate::element::ElementFormat::Default);

    assert_eq!(cli::CLI::parse_format("d"), String::from("output.txt"));
    assert_eq!(get_safe!(FORMAT), crate::element::ElementFormat::Default);

    assert_eq!(cli::CLI::parse_format("html"), String::from("output.html"));
    assert_eq!(get_safe!(FORMAT), crate::element::ElementFormat::HTML);

    assert_eq!(cli::CLI::parse_format("h"), String::from("output.html"));
    assert_eq!(get_safe!(FORMAT), crate::element::ElementFormat::HTML);

    assert_eq!(cli::CLI::parse_format("json"), String::from("output.json"));
    assert_eq!(get_safe!(FORMAT), crate::element::ElementFormat::JSON);

    assert_eq!(cli::CLI::parse_format("j"), String::from("output.json"));
    assert_eq!(get_safe!(FORMAT), crate::element::ElementFormat::JSON);

    //Test edgecases
    assert_eq!(cli::CLI::parse_format("J"), String::from("output.json"));
    assert_eq!(get_safe!(FORMAT), crate::element::ElementFormat::JSON);

    assert_eq!(cli::CLI::parse_format("HtMl"), String::from("output.html"));
    assert_eq!(get_safe!(FORMAT), crate::element::ElementFormat::HTML);

    //Test invalid params
    assert_eq!(cli::CLI::parse_format("j"), String::from("output.json"));
    assert_eq!(get_safe!(FORMAT), crate::element::ElementFormat::JSON);

    //Reset for test_element
    unsafe {
        crate::element::FORMAT = crate::element::ElementFormat::Default;
    }
}

#[test]
fn test_cli_parse_filter_style() {
    //Test valid params

    //Edited
    assert_eq!(
        cli::CLI::parse_filter_style(&String::from("edited"), Some(&String::from("true")), None),
        (0, cli::ElementFilter::Edited(true))
    );
    assert_eq!(
        cli::CLI::parse_filter_style(&String::from("edited"), Some(&String::from("false")), None),
        (0, cli::ElementFilter::Edited(false))
    );

    //Author
    assert_eq!(
        cli::CLI::parse_filter_style(
            &String::from("author"),
            Some(&String::from("==")),
            Some(&String::from("test"))
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
        (0, cli::ElementFilter::Edited(true))
    );

    assert_eq!(
        cli::CLI::parse_filter_style(&String::from("EDITED"), Some(&String::from("TruE")), None),
        (0, cli::ElementFilter::Edited(true))
    );

    assert_eq!(
        cli::CLI::parse_filter_style(&String::from("EDITED"), Some(&String::from("FAlSe")), None),
        (0, cli::ElementFilter::Edited(false))
    );

    //Test invalid params

    assert_eq!(
        cli::CLI::parse_filter_style(&String::from("asd"), None, None),
        (0, cli::ElementFilter::Default)
    );

    assert_eq!(
        cli::CLI::parse_filter_style(&String::from("E dited"), None, None),
        (0, cli::ElementFilter::Default)
    );
}
