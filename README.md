# domain_enumerator
Asynchronous domain name enumerator tool looks for available domains using a name list and the top level domain of your choice to resolve domains via brute
forcing.

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
$HOME/.cargo/domain_enumerator -n dicts/wordlist -t domain.io -o results.json

or

$HOME/.cargo/domain_enumerator -n dicts/wordlist -t domain.io
```

## License
This project is licensed under the [MIT License](https://chat.openai.com/c/LICENSE)

## References
* [Rust](https://www.rust-lang.org/)
* [Asyncronous Programming in Rust](https://rust-lang.github.io/async-book/01_getting_started/01_chapter.html)
