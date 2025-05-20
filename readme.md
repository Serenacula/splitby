# splitby

A bash script that splits each input line by a delimiter and returns a selection of the result.

## How to use

The usage format is:

```sh
splitby [options] -d <delimiter> <index_or_range>
```

The delimiter can be any regex string, e.g. `-d "\s+"`

The index states which values you want. It can accept a single number `2` or a range `2-3`. Indexes are 1-based, as standard for bash.

Negative numbers are valid, and count from the end, e.g. `-1` or `-3--1`. Mixing positive and negative is allowed, however will cause an error if the starting index is greater than the ending index.

Multiple indexes can be used, with the syntax `1 3 4-5`. The results will be separated by a new line.

### Examples

_Simple usecase_

```sh
echo "boo hoo" | splitby -d " " 1
> boo
echo "boo,hoo" | splitby -d "," 2 # You can use any delimiter you want
> hoo
```

_Range_

```sh
echo "this,is,a,test" | splitby -d "," 2-4
> is,a,test # Delimiters are kept within a range by default
```

_Negative index_

```sh
echo "this is a test" | splitby -d " " -2
> a
echo "this is a test" | splitby -d " " -3--1
> is a test
```

_Multiple indexes_

```sh
echo "this is a test" | splitby -d " " 1 3-4
> this a test
```

_Whole-input mode_

```sh
echo "this is\na test" | splitby -d " " 1 3-4
> this
> a test
```

## Installation

To install the command locally, paste the following into terminal:

```sh
curl https://raw.githubusercontent.com/Serenacula/splitby/refs/heads/main/splitby.sh > /usr/local/bin/splitby && chmod +x /usr/local/bin/splitby
```

Add `sudo` if required.

### Useful Aliases

It's also suggested to add the following aliases, for some common usecases:

```sh
alias getline="splitby --whole-string -d '\n'" # Split on newline
alias getword="splitby --skip-empty -d '\s+'" # Split on whitespace
```

These allow for fast and simple string processing, for example:

```sh
echo "this is\na test" | getline 2 | getword 2
> test
```

## Options

| Flag                                | Disable Flag            | Description                                                               |
| ----------------------------------- | ----------------------- | ------------------------------------------------------------------------- |
| -h, --help                          |                         | Print help text                                                           |
| -v, --version                       |                         | Print version number                                                      |
| -d, --delimiter \<regex>            |                         | Specify the delimiter to use (required)                                   |
| -i, --input \<input_file>           |                         | Provide an input file                                                     |
| -j, --join \<string>                |                         | Join each selection with a given string                                   |
| -w, --whole-string                  | -p, --per-line          | Processes the input as a single string, rather than each line separately  |
| --simple-ranges                     | --no-simple-ranges      | Treat ranges as a list of selections                                      |
| --replace-range-delimiter \<string> |                         | Replace the delimiters within ranges                                      |
| -c, --count                         |                         | Return the number of results after splitting                              |
| --invert                            |                         | Inverts the chosen selection                                              |
| -e, --skip-empty                    | -E, --no-skip-empty     | Skips empty fields when indexing or counting                              |
| --placeholder                       |                         | Inserts empty fields for invalid selections                               |
| -s, --strict                        | -S, --no-strict         | Shorthand for all strict features                                         |
| --strict-bounds                     | --no-strict-bounds      | Emit error if range is out of bounds                                      |
| --strict-return                     | --no-strict-return      | Emit error if there is no result                                          |
| --strict-range-order                | --no-strict-range-order | Emit error if start of a range is greater than the end (Default: enabled) |

By default the input string is taken from stdin, unless the `--input` flag is used.

Disable flags are available for making aliasing easier, allowing you to specify your preferred settings. Whichever flag was set last will be the one respected.

### MODE: Per-line

_-p, --per-line_ (default: enabled)

This functionality will have the script run once per line. Useful for when dealing with a table of information.

Note: By default, selections in this mode are joined with a space when not otherwise specified.

For example:

```
staff.csv:

Name,Age
Bob,20
Alice,30
Alex,35
```

```sh
cat staff.csv | splitby -d "," 1 # Extract just the names
> Bob
> Alice
> Alex
```

### MODE: Whole-string

_-w, --whole-string_

This treats the input as a single string. It runs once over the entire input. Useful for situations where you want to treat the string as a single blob, or you wish to use `\n` as your delimiter.

```
test-result.md:

Test results:
1. No problem
2. Error
3. No problem
4. No problem
```

```sh
cat test-result.md | splitby --whole-string -d "\n" 2 # By using \n we can select a specific line
> 2. Error
```

### Join

_-j, --join_

Normally each selection is outputted with a space between in per-line mode, or a newline in whole-string mode. This allows you to override that behaviour, replacing the default joiner with a custom string.

It does not affect delimiters within ranges unless simple-ranges is enabled.

Per-line Behaviour:

```sh
echo "this is\na test" | splitby -d " " 1 2
> this is
> a test
echo "this is\na test" | splitby -d " " --join "," 1 2
> this,is
> a,test
```

Whole-string Behaviour:

```sh
echo "this is a test" | splitby --whole-string -d " " 1 2-3 4
> this
> is a
> test
echo "this is a test" | splitby --whole-string -d " " --join "," 1 2-3 4
> this,is a,test
```

### Simple Ranges

_--simple-ranges_ | _--no-simple-ranges_

By default, if you specify a range then it will treat that as a _single selection_, outputting the entire range with delimiters. This flag will change that behaviour, so that ranges are treated as a list of individual selections.

When used with the --join flag, it will be used between each selection.

In per-line mode:

```sh
echo "this,is,a,test" | splitby -d "," 1 3-4
> this a,test
echo "this,is,a,test" | splitby -d "," --simple-ranges 1 3-4
> this a test
```

### Replace Range Delimiter

_--replace-range-delimiter_

Allows you to specify a different delimiter to use within ranges. It does not affect the functionality of --join, and is ignored when --simple-ranges is used.

```sh
echo "this is a test" | splitby -d " " 1 2-4
> this
> is a test
echo "this is a test" | splitby -d " " --replace-range-delimiter "," 1 2-4
> this
> is,a,test
```

### Count

_-c, --count_

The count option allows you to get the number of results:

```sh
echo "this;is;a;test" | splitby --count -d ";"
> 4
```

As with index selection, empty fields are counted unless you use the --skip-empty flag.

Behaviours that affect selections are ignored, e.g. --invert, --placeholder

```sh
echo "boo;;hoo" | splitby --count -d ";"
> 3
echo "boo;;hoo" | splitby --count -d ";" --skip-empty
> 2
```

### Invert

_--invert_

The invert option selects everything _except_ what you choose. Note that ranges are still in effect. You can use the --simple-ranges option if you wish each field to be treated as a single selection.

```sh
echo "this is a test" | splitby -d " " 2
> is
echo "this is a test" | splitby -d " " --invert 2
> this a test
```

### Skip-empty

_-e, --skip-empty_ | _-E, --no-skip-empty_

By default the script does not skip empty values. --skip-empty tells it to ignore empty fields when counting and indexing.

With indexes:

```sh
echo "boo,,hoo" | splitby -d "," 2
>
echo "boo,,hoo" | splitby -d "," 2 --skip-empty
> hoo
```

With count:

```sh
echo "boo,,hoo" | splitby -d "," --count
> 3
echo "boo,,hoo" | splitby -d "," --count --skip-empty
> 2
```

### Placeholder

_--placeholder_

This is a somewhat niche flag for the situation where you need a reliable output format. Normally, an invalid selection is skipped, however with this flag an invalid selection will output an empty string instead.

A join string is added here for clarity:

```sh
echo "boo hoo foo" | splitby -d " " -j ":" 1 4 2 # Out of range value gets skipped
> boo:hoo
echo "boo hoo foo" | splitby -d " " --placeholder 1 4 2
> boo::hoo
```

### Strict

_-s, --strict_ | _-S, --no-strict_

Strict controls whether the program will fail silently or explicitly when encountering errors. Both can be useful in different situations.

There are several modes available:

#### Strict Bounds

_--strict-bounds_ | _--no-strict-bounds_ (default: disabled)

In normal operation, the script silently limits the bounds to within the range. --strict-bounds tells it to emit an error instead.

For example, this is silently corrected to `2-3`. With strict mode, it emits an error to stderr instead:

```sh
echo "boo hoo foo" | splitby -d " " 2-5
> hoo foo
echo "boo hoo foo" | splitby -d " " --strict-bounds 2-5
> End index (5) out of bounds. Must be between 1 and 3
```

This also applies to single indexes out of bounds. By default, they emit an empty line:

```sh
echo "boo hoo foo" | splitby -d " " 4
>
echo "boo hoo foo" | splitby -d " " --strict-bounds 4
> Index (4) out of bounds. Must be between 1 and 3
```

#### Strict Result

_--strict-return_ | _--no-strict-return_ (default: disabled)

In situations where there is no results at all, the script defaults to emitting nothing. --strict-return tells it to emit an error instead.

For example if a delimiter has no results:

```sh
echo "boo hoo" | splitby -d ","
>
echo "boo hoo" | splitby -d "," --strict-return
> strict return check failed: No valid fields available
```

Similarly, if you skip empty fields:

```sh
echo ",," | splitby --skip-empty -d ","
>
echo ",," | splitby --skip-empty -d "," --strict-return
> strict return check failed: No valid fields available
```

It has no effect when --count is used.

#### Strict Range Order

_--strict-range-order_ | _--no-strict-range-order_ (default: enabled)

This flag causes an error to emit if the start of a range is after the end, e.g. `3-1`.

```sh
echo "boo hoo" | splitby -d " " 3-1
> End index (1) is less than start index (3) in selection 3-1
echo "boo hoo" | splitby -d " " --no-strict-range-order 3-1
>
```
