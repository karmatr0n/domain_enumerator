# domain_resolver
Asynchronous domain resolver that looks for available domains using a word list and the top level domain of your choice.

## How to build this
```
cargo build --release
```

## How to install this
```
cargo install --path .
```

## How to run this
```
$HOME/.cargo/bin/domain_resolver -w dicts/wordlist -t io

or

$HOME/.cargo/bin/domain_resolver --help
```

## License
This project is licensed under the [MIT License](https://chat.openai.com/c/LICENSE)

## References
* [Rust](https://www.rust-lang.org/)
* [Asyncronous Programming in Rust](https://rust-lang.github.io/async-book/01_getting_started/01_chapter.html)
