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
alias getline="splitby -d '\n'"
alias getword="splitby -d '\s+'"
```

These allow for fast and simple string processing, for example:

```sh
echo "this is\na test" | getline 2 | getword 2
> test
```

## Options

| Flag                        | Description                                      |
| --------------------------- | ------------------------------------------------ |
| -h, --help                  | Print help text                                  |
| -v, --version               | Print version number                             |
| -d, --delimiter \<regex>    | Specify the delimiter to use (required)          |
| -i, --input \<input_string> | Provide input string directly                    |
| -c, --count                 | Return the number of results after splitting     |
| -s, --strict                | Shorthand for --strict-bounds and --strict-empty |
| -sb, --strict-bounds        | Emit error if range is out of bounds             |
| -se, --strict-empty         | Emit error if there is no result                 |
| -e, --skip-empty            | Skips empty fields when indexing or counting     |

By default the input string is taken from stdin, unless the `--input` flag is used.

### Count

The count option allows you to get the number of results, useful for scripting:

```sh
echo "this;is;a;test" | splitby --count -d ";"
> 4
```

**As with index selection, empty fields are counted** unless you use the --skip-empty flag

```sh
echo "boo;;hoo" | splitby --count -d ";"
> 3
echo "boo;;hoo" | splitby --count --skip-empty -d ";"
> 2
```

### Strict-bounds

In normal operation, the script silently limits the bounds to within the range. --strict-bounds tells it to emit an error instead.

For example, this is silently corrected to `2-3`. With strict mode, it emits an error to stderr instead:

```sh
echo "boo hoo foo" | splitby -d " " 2-5
> hoo foo
echo "boo hoo foo" | splitby --strict-bounds -d " " 2-5
> End index (5) out of bounds. Must be between 1 and 3
```

This also applies to single indexes out of bounds. By default, they emit an empty line:

```sh
echo "boo hoo foo" | splitby --strict-bounds -d " " 4
>
echo "boo hoo foo" | splitby --strict-bounds -d " " 4
> Index (4) out of bounds. Must be between 1 and 3
```

### Strict-empty

In situations where there is no results at all, the script defaults to emitting nothing. --strict-empty tells it to empty an error instead.

For example if a delimiter has no results:

```sh
echo "boo hoo" | splitby -d ","
>
echo "boo hoo" | splitby --strict-empty -d ","
> Strict empty check failed: No valid fields available
```

Similarly, if you skip empty fields:

```sh
echo ",," | splitby --skip-empty -d ","
>
echo ",," | splitby --strict-empty -d ","
> Strict empty check failed: No valid fields available
```

It has no effect when --count is used.

### Skip-empty

By default the script does not skip empty values. --skip-empty tells it to ignore empty fields when counting and indexing.

For example, the default behaviour is this:

```sh
echo "boo,,hoo" | splitby -d "," 2
>
echo "boo,,hoo" | splitby -d "," --count
> 3
```

With --skip-empty this becomes:

```sh
echo "boo,,hoo" | splitby --skip-empty -d "," 2
> hoo
echo "boo,,hoo" | splitby --skip-empty -d "," --count
> 2
```
