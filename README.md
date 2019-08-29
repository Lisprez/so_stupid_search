# so stupid search tool

# install

## install from source code
* Download the file
* Enter the directory contains file with name of "Cargo.toml"
* Run command as blow:

```bash
cargo build --release
```

* Then you will get the executable file ./target/release/sfind

## install by cargo

```bash
cargo install so_stupid_search
alias sss=$HOME/.cargo/bin/so_stupid_search
```

# Usage

  ```bash
  sss search-string start-directory
  ```

# Example

## common search
```bash
sss walk .
```

## search specified type of file

```bash
#only search in kubernetes yaml files
sss -t yaml fuck_str .
```
 
