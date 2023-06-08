# domain_resolver
asynchronous domain resolver looks for available domains using a word list and the top level domain of your choice.

## How to build this
```
cargo build --release
```


## How to run this
```
./target/release/domain_resolver dicts/wordlist io
```

## How to install this
```
cargo install --path .
```

## License
This project is licensed under the [MIT License](https://chat.openai.com/c/LICENSE)

## References
* [Rust](https://www.rust-lang.org/)
* [Asyncronous Programming in Rust](https://rust-lang.github.io/async-book/01_getting_started/01_chapter.html)
