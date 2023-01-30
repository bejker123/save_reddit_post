![Build Badge](https://github.com/bejker123/save_reddit_post/actions/workflows/build.yml/badge.svg)
![Tests Badge](https://github.com/bejker123/save_reddit_post/actions/workflows/tests.yml/badge.svg)

# SRP

**Save Reddit Post** is a program made to easily save any reddit post, without any authentication. 

## Installation

First make sure you have [rust](https://www.rust-lang.org/tools/install) installed.
Then simply clone this repository:

```bash
git clone https://github.com/bejker123/save_reddit_post
```

## Usage
Use this as any other rust cli program.
```bash
#You can build it first:
cargo build
#Then run it:
target/debug/srp #arguments go here
#Or run it directly
cargo run -- #pass arguments after "--"
#for help use -h/--help argument
```

## Note
Please note that this repo is in its early stages.
It will go through many changes in the future!

## Roadmap

- [ ] Change first arg to url?
- [ ] Add getting all comments on a post 
- [ ] Add formatting, ex. HTML


## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate. At this moment, this is not applicable, since no tests are present.

## License
[MIT](https://choosealicense.com/licenses/mit/)