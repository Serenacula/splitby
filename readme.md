# splitby

A bash script that splits a string by a delimiter and returns a selection of the result

## How to use

The usage format is: `splitby [options] -d <delimiter> <index_or_range>`

The delimiter is a regex string, e.g. "\s+"

The range states which entries you want in the output. It accepts a specific index or a range. A range can be left open, e.g. `-3` will go from the start to the third item.

Examples:

`echo "boo hoo" | splitby -d " " 2` will output `hoo`

`echo "boo hoo foo" | splitby -d " " 2-3` will output `hoo foo`

`echo "this is a test" | splitby -d " " 2-` will output `is a test`
