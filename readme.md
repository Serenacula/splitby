# splitby

A bash script that splits a string by a delimiter and returns a selection of the result.

## How to use

The usage format is:

```sh
splitby [options] -d <delimiter> <index_or_range>
```

The delimiter is any regex string, e.g. `-d "\s+"`

The index states which values you want. It can accept a single number `2` or a range `2-3`.

Negative numbers are valid, and count from the end. `-1` or `-3--1`. Mixing positive and negative is allowed, however will cause an error if the starting index is after the ending index.

Multiple indexes can be used, with the syntax `1 3 4-5`. The results will be separated by a new line.

### Examples

_Simple usecase_

```sh
echo "boo hoo" | splitby -d " " 1
> boo
echo "boo,hoo" | splitby -d "," 2
> hoo
```

_Range_

```sh
echo "boo hoo foo" | splitby -d " " 2-3 # Delimiter is kept within a range
> hoo foo
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
> this
> a test
```

### Empty Fields

**Empty fields are considered valid!**

The following will treat the empty space as a valid field when indexing:

```sh
echo "boo,,hoo" | splitby -d "," 2-3
> ,foo
```

If you wish to skip it, you can do so by altering the regex, or by using the --skip-empty flag:

```sh
echo "boo,,hoo" | splitby -d ",+" 2
> foo
echo "boo,,hoo" | splitby --skip-empty -d "," 2
> foo
```

## Installation

To install the command locally, paste the following into terminal:

```sh
curl https://raw.githubusercontent.com/Serenacula/splitby/refs/heads/main/splitby.sh > /usr/local/bin/splitby && chmod +x /usr/local/bin/splitby
```

### Useful Aliases

It's also suggested to add the following aliases, for some common usecases:

```sh
alias getline="splitby -d '\n'" # Split on newline
alias getword="splitby -d '\s+'" # Split on whitespace
```

These allow for fast and simple string processing, for example:

```sh
echo "this is\na test" | getline 2 | getword 2
> test
```

## Options

| Flag                      | Disable Flag            | Description                                                             |
| ------------------------- | ----------------------- | ----------------------------------------------------------------------- |
| -h, --help                |                         | Print help text                                                         |
| -v, --version             |                         | Print version number                                                    |
| -d, --delimiter \<regex>  |                         | Specify the delimiter to use (required)                                 |
| -i, --input \<input_file> |                         | Provide an input file                                                   |
| -c, --count               |                         | Return the number of results after splitting                            |
| -e, --skip-empty          | -E, --no-skip-empty     | Skips empty fields when indexing or counting                            |
| -s, --strict              | -S, --no-strict         | Shorthand for all strict features                                       |
| --strict-bounds           | --no-strict-bounds      | Emit error if range is out of bounds                                    |
| --strict-return           | --no-strict-return      | Emit error if there is no result                                        |
| --strict-range-order      | --no-strict-range-order | Emit error if the start of a range is before the end (Default: enabled) |

By default the input string is taken from stdin, unless the `--input` flag is used.

Disable flags are available for making aliasing easier, allowing you to specify your preferred settings. Whichever flag was set last will be the one respected.

### Count

_-c, --count_

The count option allows you to get the number of results:

```sh
echo "this;is;a;test" | splitby --count -d ";"
> 4
```

As with index selection, empty fields are counted unless you use the --skip-empty flag.

```sh
echo "boo;;hoo" | splitby --count -d ";"
> 3
echo "boo;;hoo" | splitby --count -d ";" --skip-empty
> 2
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

### Strict

_-s, --strict_ | _-S, --no-strict_

Strict controls whether the program will fail silently or explicitly when encountering errors. Both can be useful in different situations.

There are several mode available:

#### Strict Bounds

_--strict-bounds_ | _--no-strict-bounds_ (default: disabled)

In normal operation, the script silently limits the bounds to within the range. --strict-bounds tells it to emit an error instead.

For example, this is silently corrected to `2-3`. With strict mode, it emits an error to stderr instead:

```sh
echo "boo hoo foo" | splitby -d " " 2-5
> hoo foo
echo "boo hoo foo" | splitby -d " " 2-5  --strict-bounds
> End index (5) out of bounds. Must be between 1 and 3
```

This also applies to single indexes out of bounds. By default, they emit an empty line:

```sh
echo "boo hoo foo" | splitby -d " " 4
>
echo "boo hoo foo" | splitby -d " " 4  --strict-bounds
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
> End index (1) is less than start index (3) in selection 3-1\n
echo "boo hoo" | splitby -d " " 3-1 --no-strict-range-order
>
```
