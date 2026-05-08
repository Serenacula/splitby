# splitby

![Splitby](/docs/public/og.webp)

A high-performance Rust command-line tool that splits text by a regex delimiter and returns selected parts of the result. A powerful, multi-threaded alternative to `cut` with regex support.

Full documentation: **[serenacula.github.io/splitby](https://serenacula.github.io/splitby/)**

## Installation

**Homebrew**

```sh
brew install serenacula/tap/splitby
```

**Cargo**

```sh
cargo install splitby
```

Or install from [releases](https://github.com/Serenacula/splitby/releases)

## Usage

```sh
splitby <delimiter> [selections] [options]
```

Selections are 1-based indexes or ranges. Negative indexes count from the end.

```sh
echo "boo,hoo,foo" | splitby , 2
> hoo

echo "this is a test" | splitby " " 2-4
> is a test

echo "this is a test" | splitby " " -2
> a

echo "a:b:c:d" | splitby : first last
> a
> d

echo "this,is,a,test" | splitby , 2 --invert
> this,a,test

echo -e "apple,banana,cherry\na,bb,ccc" | splitby , --align
> apple,banana,cherry
> a    ,bb    ,ccc
```

Regex delimiters are wrapped in `/…/`:

```sh
echo "one  two   three" | splitby "/\s+/" 1 3
> one three
```

## Options

| Flag                              | Disable Flag                 | Description                                                                  | Default  |
| --------------------------------- | ---------------------------- | ---------------------------------------------------------------------------- | -------- |
| `-h, --help`                      |                              | Print help text                                                              |          |
| `-v, --version`                   |                              | Print version number                                                         |          |
| `--input=<FILE>`                  |                              | Provide an input file                                                        |          |
| `--output=<FILE>`                 |                              | Write output to a file                                                       |          |
| `-d, --delimiter=<REGEX>`         |                              | Specify the delimiter                                                        |          |
| `-j, --join=<STRING\|HEX>`        |                              | Join selections with a given string                                          |          |
| `-p, --placeholder=<STRING\|HEX>` |                              | Insert placeholder for out-of-bounds selections                              |          |
| `-t, --terminator=<STRING\|HEX>`  |                              | Replace the output record terminator                                         |          |
| `--per-line`                      |                              | Process input line by line (default)                                         | Enabled  |
| `-w, --whole-string`              |                              | Process input as a single string                                             |          |
| `-z, --zero-terminated`           |                              | Process input as zero-terminated strings                                     |          |
| `-f, --fields`                    |                              | Select fields split by delimiter (default)                                   | Enabled  |
| `-b, --bytes`                     |                              | Select bytes from the input                                                  |          |
| `-c, --characters`                |                              | Select grapheme clusters from the input                                      |          |
| `-a, --align[=MODE]`              |                              | Align fields to consistent column widths (`left`, `right`, `squash`, `none`) | Disabled |
| `--count`                         |                              | Return the number of fields after splitting                                  |          |
| `-i, --invert`                    |                              | Invert the selection                                                         |          |
| `-e, --skip-empty-fields`         | `-E, --no-skip-empty-fields` | Skip empty fields when indexing or counting                                  | Disabled |
| `-l, --skip-empty-lines`          | `-L, --no-skip-empty-lines`  | Suppress output records whose result is empty                                | Disabled |
| `-s, --skip-undelimited`          | `-S, --no-skip-undelimited`  | Suppress records with no delimiter (fields mode only)                        | Disabled |
| `--strict`                        | `--no-strict`                | Shorthand for all strict features                                            |          |
| `--strict-bounds`                 | `--no-strict-bounds`         | Error if selection is out of bounds                                          | Disabled |
| `--strict-return`                 | `--no-strict-return`         | Error if result is empty                                                     | Disabled |
| `--strict-range-order`            | `--no-strict-range-order`    | Error if range start is greater than end                                     | Enabled  |
| `--strict-utf8`                   | `--no-strict-utf8`           | Error on invalid UTF-8 sequences                                             | Disabled |
