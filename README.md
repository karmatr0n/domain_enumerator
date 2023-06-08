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
$HOME/.cargo/domain_resolver -n dicts/wordlist -t io -o results.json

or

$HOME/.cargo/domain_resolver -n dicts/wordlist -t io
```

## License
This project is licensed under the [MIT License](https://chat.openai.com/c/LICENSE)

## References
* [Rust](https://www.rust-lang.org/)
* [Asyncronous Programming in Rust](https://rust-lang.github.io/async-book/01_getting_started/01_chapter.html)
