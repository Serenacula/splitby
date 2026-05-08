# splitby

![Splitby](/docs/public/og.webp)

A high-performance Rust command-line tool that splits text by a regex delimiter and returns selected parts of the result. A powerful, multi-threaded alternative to `cut` with regex support.

## How to use

The usage format is:

```sh
splitby <delimiter> [options] [selections]
```

The delimiter can be any regex string (wrapped in `/.../`) or a literal string, e.g. `"/\\s+/"` for regex or `","` for literal.

The selection states which values you want. It can accept a single number `2` or a range `2-3`. Indexes are 1-based, as standard for Unix text tools like `cut` and `awk`.

Negative numbers are valid, and count from the end, e.g. `-1` or `-3--1`. Mixing positive and negative is allowed, however will cause an error if the starting index is greater than the ending index.

You can also use special keywords: `start` or `first` (equivalent to `1`), and `end` or `last` (equivalent to `-1`). These can be used in ranges like `first-last` or `start-2`.

Multiple indexes can be used, with the syntax `1 3 4-5`. By default, selections are joined by the delimiter between them.

### Examples

_Simple usecase_

```sh
echo "boo hoo" | splitby " " 1
> boo
echo "boo,hoo" | splitby , 2
> hoo
```

_Regex_

```sh
echo -e "boo hoo\n  foo" | splitby "/\\s+/" -w 1 3
> boo foo # by default, the delimiter after the previous selection is kept between selections
```

_Range_

```sh
echo "this,is,a,test" | splitby , 2-4
> is,a,test
```

_Negative index_

```sh
echo "this is a test" | splitby " " -2
> a
echo "this is a test" | splitby " " -3--1
> is a test
```

_Multiple indexes_

```sh
echo "this is a test" | splitby " " 1 3-4
> this a test
```

_Whole-input mode_

```sh
echo -e "line1\nline2\nline3" | splitby "/\n/" -w 2
> line2
```

_Character mode_

```sh
echo "café" | splitby -c 1 4
> cé # Character mode selects specific characters, rather than fields
```

_Special keywords_

```sh
echo "this is a test" | splitby " " first-last
> this is a test
echo "this is a test" | splitby " " start-2
> this is
echo "this is a test" | splitby " " end
> test
```

## Installation

**Homebrew** (macOS/Linux):

```sh
brew install serenacula/tap/splitby
```

Or you can find binaries to install under [releases](https://github.com/Serenacula/splitby/releases).

Alternatively, you can build from source if you prefer:

1. Install Rust via [rustup](https://rustup.rs)
2. `git clone https://github.com/serenacula/splitby`
3. `cargo build --release`
4. `mv ./target/release/splitby /usr/local/bin/`

### Useful Aliases

It's also suggested to add the following aliases to your .bashrc or .zshrc, for some common usecases:

```sh
alias getline="splitby -w '/\n/'" # Split on newline
alias getword="splitby -e '/\s+/'" # Split on whitespace (regex), skipping empty fields
```

These allow for fast and simple string processing:

```sh
echo -e "line1\nline2\nline3" | getline 1
> line1
```

Or quick table processing:

```
file.txt:

Item Value
Apple 1.5
Pear 1.3
Car 30000
```

```sh
cat file.txt | getword 1
> Item
> Apple
> Pear
> Car
```

## Options

| Flag                          | Disable Flag              | Description                                                              | Default Value |
| ----------------------------- | ------------------------- | ------------------------------------------------------------------------ | ------------- |
| `-h, --help`                  |                           | Print help text                                                          |               |
| `-v, --version`               |                           | Print version number                                                     |               |
| `--input=<FILE>`              |                           | Provide an input file                                                    |               |
| `--output=<FILE>`             |                           | Write output to a file                                                   |               |
| `-d, --delimiter=<REGEX>`     |                           | Specify the delimiter to use                                             |               |
| `-j, --join=<STRING\|HEX>`    |                           | Join each selection with a given string                                  |               |
| `-p, --placeholder=<STRING\|HEX>` |                       | Inserts placeholder for invalid selections                               |               |
| `-t, --terminator=<STRING\|HEX>` |                        | Replace the output record terminator                                     |               |
| `--per-line`                  |                           | Processes the input line by line (default)                               | Enabled       |
| `-w, --whole-string`          |                           | Processes the input as a single string, rather than each line separately |               |
| `-z, --zero-terminated`       |                           | Processes the input as zero-terminated strings                           |               |
| `-f, --fields`                |                           | Select fields split by delimiter (default)                               | Enabled       |
| `-b, --bytes`                 |                           | Select bytes from the input                                              |               |
| `-c, --characters`            |                           | Select characters from the input                                         |               |
| `-a, --align[=MODE]`          |                           | Align fields to consistent column widths; mode defaults to `left` (`left`, `right`, `squash`, `none`) | Disabled      |
| `--count`                     |                           | Return the number of results after splitting                             |               |
| `-i, --invert`                |                           | Inverts the chosen selection                                             |               |
| `-e, --skip-empty-fields`     | `-E, --no-skip-empty-fields` | Skips empty fields when indexing or counting                          | Disabled      |
| `-l, --skip-empty-lines`      | `-L, --no-skip-empty-lines`  | Suppresses output records whose result is empty                       | Disabled      |
| `-s, --skip-undelimited`      | `-S, --no-skip-undelimited`  | Suppresses records with no delimiter (fields mode only)               | Disabled      |
| `--strict`                    | `--no-strict`             | Shorthand for all strict features                                        |               |
| `--strict-bounds`             | `--no-strict-bounds`      | Emit error if range is out of bounds                                     | Disabled      |
| `--strict-return`             | `--no-strict-return`      | Emit error if there is no result                                         | Disabled      |
| `--strict-range-order`        | `--no-strict-range-order` | Emit error if start of a range is greater than the end                   | Enabled       |
| `--strict-utf8`               | `--no-strict-utf8`        | Emit error on invalid UTF-8 sequences                                    | Disabled      |

By default the input string is taken from stdin, unless the `--input` flag is used.

Disable flags are available for making aliasing easier, allowing you to specify your preferred settings. Flags respect last-flag-wins logic.

### Delimiter

The delimiter is passed as a plain argument — no flag needed:

```sh
echo "this,is a.test" | splitby "/[,.]/" --strict 1 3
> this,test
```

Flags can go anywhere relative to the delimiter and selections. The first argument that isn't a flag or a selection is taken as the delimiter.

The `-d` / `--delimiter` flag exists for the cases where your delimiter would otherwise be ambiguous — for example, if it looks like a number (and would be parsed as a selection) or starts with `-` (and would be parsed as a flag):

```sh
echo "1 2 3" | splitby -d "1" 2   # delimiter is "1"; without -d it would be parsed as a selection
echo "a-b-c" | splitby -d "-" 2   # delimiter is "-"; without -d it would error as an invalid flag
```

### Input Modes

#### MODE: Per-line

_--per-line_ (default: enabled)

This processes the input line by line. Useful for when dealing with a table of information.

For example:

```
staff.csv:

Name,Age
Bob,20
Alice,30
Alex,35
```

```sh
cat staff.csv | splitby , 1 # Extract just the names
> Name
> Bob
> Alice
> Alex
```

#### MODE: Whole-string

_-w, --whole-string_

This treats the input as a single string. It runs once over the entire input. Useful for situations where you want to treat the string as a single blob, or you wish to use `\n` as your delimiter.

```sh
echo "a,b,c" | splitby "," -w 2 # Process entire input as one string
> b
```

#### MODE: Zero-terminated

_-z, --zero-terminated_

This mode treats the input as a sequence of zero-terminated strings. It runs once over the entire input. Useful for processing filenames from `find -print0` or other tools that output null-terminated strings.

```sh
# split on /, join with \n, and get the last field
find . -name "*.txt" -print0 | splitby "/" -j "\n" -z last
> file1.txt
> file2.txt
> file3.txt
```

### Selection Modes

#### MODE: Fields

_-f, --fields_ (default: enabled)

This mode treats the input as a sequence of fields, split by a delimiter.

```sh
echo "this is a test" | splitby " " 2
> is
```

#### MODE: Chars

_-c, --characters_

This mode treats the input as a sequence of characters. Useful for situations where you need to work with a sequence of characters.

Note: Unlike `cut`, this respects visible characters, rather than byte counts.

```sh
echo "café" | splitby -c 3-4
> fé
```

#### MODE: Bytes

_-b, --bytes_

This mode treats the input as a sequence of bytes.

Note: Join is not supported in bytes mode.

```sh
echo "this is a test" | splitby -b 2-14
> his is a test
```

### Selection Options

#### Invert

_--invert_

The invert option selects everything _except_ what you choose.

```sh
echo "this is a test" | splitby " " 2
> is
echo "this is a test" | splitby " " --invert 2
> this a test
```

#### Skip-empty-fields

_-e, --skip-empty-fields_ | _-E, --no-skip-empty-fields_ (default: disabled)

By default the tool does not skip empty values. `--skip-empty-fields` tells it to ignore empty fields when counting and indexing.

With indexes:

```sh
echo "boo,,hoo" | splitby , 2
>
echo "boo,,hoo" | splitby , --skip-empty-fields 2
> hoo
```

#### Skip-empty-lines

_-l, --skip-empty-lines_ | _-L, --no-skip-empty-lines_ (default: disabled)

Suppresses output records whose result is empty after processing. Useful when a dataset has blank lines that would otherwise produce empty output.

```sh
printf "a,b\n\nc,d\n" | splitby "," -l 1
> a
> c
```

#### Skip-undelimited

_-s, --skip-undelimited_ | _-S, --no-skip-undelimited_ (default: disabled)

Suppresses records that contain no delimiter at all. Only available in fields mode.

```sh
printf "a,b\nno-delimiter\nc,d\n" | splitby "," -s 1
> a
> c
```

### Transform Options

#### Align

_-a, --align_

This option pads fields so that columns line up across lines. Selections are optional; omitting them returns all fields.

It accepts an optional mode:

- `left` (default): fields are left-aligned within their column
- `right`: fields are right-aligned within their column
- `squash`: padding is placed after the delimiter, aligning the first character of each field

```sh
echo -e "apple,banana,cherry\na,b,c" | splitby -a ,
> apple,banana,cherry
> a    ,b     ,c

echo -e "apple,banana,cherry\na,b,c" | splitby , --align=right
> apple,banana,cherry
>     a,     b,     c

echo -e "apple,banana,cherry\na,b,c" | splitby , --align=squash
> apple,banana,cherry
> a,    b,    c
```

#### Join

_-j \<STRING|HEX\>, --join=\<STRING|HEX\>_

This flag lets you control how selections are joined together.

By default, the joiner is the delimiter after the previous selection. If unavailable, the joiner is the delimiter before the next selection. If both are unavailable, the joiner is the first delimiter in the record.

```sh
echo "this is\na test" | splitby " " 1 2
> this is
> a test
echo "this is\na test" | splitby " " --join="," 1 2
> this,is
> a,test
```

The join flag also accepts hex values (with `0x` or `0X` prefix) for multi-byte joiners or non-printable characters:

```sh
echo "this is\na test" | splitby " " --join="0x2C20" 1 2
> this, is
> a, test
```

There are also a number of useful keywords you can use (only in fields mode):
| Keyword | Description |
|-------------------|-----------------------------------------------------|
| `--join=auto` | Automatically tries `after-previous`, then `before-next`, then `space` |
| `--join=after-previous` | Use the delimiter after the previous selection |
| `--join=before-next` | Use the delimiter before the next selection |
| `--join=first` | Use the first delimiter in the record |
| `--join=last` | Use the last delimiter in the record |
| `--join=space` | Use a space character |
| `--join=none` | No join (equivalent to "") |

#### Placeholder

_-p, --placeholder=\<STRING|HEX\>_

This is a useful flag for the situation where you need a reliable output format. Normally an invalid selection is skipped, however with this flag an invalid selection will output the given placeholder string instead.

The placeholder accepts both string values and hex values (with `0x` or `0X` prefix). Hex values are useful for multi-byte placeholders or non-printable characters.

A join string is added here for clarity:

```sh
echo "boo hoo foo" | splitby " " -j ":" 1 4 2 # Out of range value gets skipped
> boo:hoo
echo "boo hoo foo" | splitby " " -j ":" --placeholder="?" 1 4 2
> boo:?:hoo
echo "boo hoo foo" | splitby " " -j "," --placeholder="" 1 4 2
> boo,,hoo # empty string placeholder
echo "boo hoo foo" | splitby " " -j "," --placeholder="0x2C20" 1 4 2
> boo,, ,hoo # hex placeholder (0x2C20 = ", " in UTF-8)
```

#### Terminator

_-t \<STRING|HEX\>, --terminator=\<STRING|HEX\>_

Replaces the record terminator written after each output record. In per-line mode the default is `\n`; in zero-terminated mode it is `\0`; in whole-string mode it appends once after the output.

```sh
printf "a,b\nc,d\n" | splitby "," --terminator="|" 1
> a|c|

printf "a,b\nc,d\n" | splitby "," --terminator="" 1
> ac  # empty terminator concatenates records
```

Accepts hex values for non-printable characters:

```sh
printf "a,b\nc,d\n" | splitby "," --terminator=0x0d0a 1  # CRLF line endings
```

The terminator is only appended when the original record had one — a final line without a trailing newline stays unterminated.

### Count

_--count_

The count option allows you to get the number of results:

```sh
echo "this;is;a;test" | splitby ";" --count
> 4
```

As with index selection, empty fields are counted unless you use the `--skip-empty-fields` flag.

Behaviours that affect selections are ignored, e.g. `--invert`, `--placeholder`

```sh
echo "boo;;hoo" | splitby ";" --count
> 3
echo "boo;;hoo" | splitby ";" --count --skip-empty-fields
> 2
```

With count:

```sh
echo "boo,,hoo" | splitby , --count
> 3
echo "boo,,hoo" | splitby , --count --skip-empty-fields
> 2
```

### Strictness Options

#### Strict

_--strict_ | _--no-strict_

The plain `--strict` flag is shorthand for all strictness options listed below.

#### Strict Bounds

_--strict-bounds_ | _--no-strict-bounds_ (default: disabled)

In normal operation, the tool silently limits the bounds to within the range. `--strict-bounds` tells it to emit an error instead.

For example, this is silently corrected to `2-3`. With strict mode, it emits an error to stderr instead:

```sh
echo "boo hoo foo" | splitby " " 2-5
> hoo foo
echo "boo hoo foo" | splitby " " --strict-bounds 2-5
> line 1: strict-bounds error: end index (5) out of bounds, must be between 1 and 3
```

This also applies to single indexes out of bounds.

```sh
echo "boo hoo foo" | splitby " " 4
> # Empty output (index out of bounds)
echo "boo hoo foo" | splitby " " --strict-bounds 4
> line 1: strict-bounds error: index (4) out of bounds, must be between 1 and 3
```

#### Strict Return

_--strict-return_ | _--no-strict-return_ (default: disabled)

In situations where the selected result would be empty, the tool defaults to emitting nothing. `--strict-return` tells it to emit an error instead.

For example:

```sh
echo ",boo" | splitby , 1
> # Empty output (field 1 is empty)
echo ",boo" | splitby , --strict-return 1
> line 1: strict-return error: no valid output
```

Similarly, if you skip empty fields:

```sh
echo ",," | splitby , --skip-empty-fields
> # Empty output (all fields are empty)
echo ",," | splitby , --skip-empty-fields --strict-return
> line 1: strict-return error: empty field
```

It has no effect when `--count` is used.

#### Strict Range Order

_--strict-range-order_ | _--no-strict-range-order_ (default: enabled)

This flag causes an error to emit if the start of a range is after the end, e.g. `3-1`.

```sh
echo "boo hoo" | splitby " " 3-1
> line 1: strict-range-order error: end index (1) is less than start index (3) in selection 3-1
echo "boo hoo" | splitby " " --no-strict-range-order 3-1
> # No error emitted
```

#### Strict UTF-8

_--strict-utf8_ | _--no-strict-utf8_ (default: disabled)

By default, when the tool encounters invalid UTF-8 sequences, it replaces them with the Unicode replacement character (U+FFFD). When `--strict-utf8` is enabled, the tool will emit an error instead of silently replacing invalid sequences.

This is particularly useful when processing binary data or when you need to ensure data integrity.

```sh
# Invalid UTF-8 sequence (example)
echo -ne "hello\xFFworld" | splitby -c 1-5
> hello # Replacement character used, but only first 5 characters returned
echo -ne "hello\xFFworld" | splitby --strict-utf8 -c 1-5
> line 1: strict-utf8 error: input is not valid UTF-8
```
