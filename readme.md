# splitby

A bash script that splits a string by a delimiter and returns a selection of the result

## How to use

The usage format is: `splitby [options] -d <delimiter> <index_or_range>`

The delimiter is any regex string, e.g. `-d "\s+"`

The range states which entries you want in the output. It accepts a specific index or a range. A range can be left open, e.g. `-3` will go from the start to the third item.

### Examples

`echo "boo hoo" | splitby -d " " 2` will output `hoo`

`echo "boo hoo foo" | splitby -d " " 2-3` will output `hoo foo`

`echo "this is a test" | splitby -d " " 2-` will output `is a test`

### Count

The count option allows you to get the number of results, useful for scripting:

`echo "this is a test" | splitby --count -d " "` will output `4`

## Installation

To install the command locally, paste the following into terminal:

`curl https://raw.githubusercontent.com/Serenacula/splitby/refs/heads/main/splitby.sh > /usr/local/bin/splitby && chmod +x /usr/local/bin/splitby`

### Useful Aliases

It's also suggested to add the following aliases, for some common usecases:

`alias getline="splitby -d '\n'"`

`alias getword="splitby -d '\s+'"`

These allow for fast and simple string processing, for example:

`echo "this is\na test" | getline 2 | getword 2` outputs `test`

## Options

```
-d, --delimiter <regex>       Specify the delimiter to use (required)
-i, --input <input_string>    Provide input string directly
-c, --count                   Return the number of results after split
-s, --strict-bounds           Emit error if range is out of bounds (default: disabled)
```

By default the input string is taken from stdin, unless the `--input` flag is used.
