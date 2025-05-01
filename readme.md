# splitby

A bash script that splits a string by a delimiter and returns a selection of the result

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
echo "boo hoo" | splitby -d " " 2
> hoo
```

_Range_

```sh
echo "boo,hoo,foo" | splitby -d "," 2-3
> hoo,foo
```

_Negative index_

```sh
echo "this is a test" | splitby -d " " -2
> a
```

_Multiple indexes_

```sh
echo "this is a test" | splitby -d " " 1 3-4
> this
> a test
```

### Count

The count option allows you to get the number of results, useful for scripting:

```sh
echo "this;is;a;test" | splitby --count -d ";"
> 4
```

### Strict-bounds

In normal operation, the script silently limits the bounds to within the range. Strict mode tells it to emit an error instead.

For example, this is silently corrected to `2-3`:

```sh
echo "boo hoo foo" | splitby -d " " 2-5
> hoo foo
```

This is also true for single indexes, when they are out of bounds. This becomes `3`:

```sh
echo "boo hoo foo" | splitby -d " " 4
> foo
```

In both cases, strict mode will instead emit an error.

## Installation

To install the command locally, paste the following into terminal:

```sh
curl https://raw.githubusercontent.com/Serenacula/splitby/refs/heads/main/splitby.sh > /usr/local/bin/splitby && chmod +x /usr/local/bin/splitby
```

### Useful Aliases

It's also suggested to add the following aliases, for some common usecases:

```sh
alias getline="splitby -d '\n'"
alias getword="splitby -d '\s+'"
```

These allow for fast and simple string processing, for example:

```sh
echo "this is\na test" | getline 2 | getword 2
> test
```

## Options

| Flag                        | Description                                              |
| --------------------------- | -------------------------------------------------------- |
| -d, --delimiter \<regex>    | Specify the delimiter to use (required)                  |
| -i, --input \<input_string> | Provide input string directly                            |
| -c, --count                 | Return the number of results after splitting             |
| -s, --strict-bounds         | Emit error if range is out of bounds (default: disabled) |

By default the input string is taken from stdin, unless the `--input` flag is used.
