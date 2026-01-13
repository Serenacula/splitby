# Planned Features

## Core Features

-   Ranged indexes (done)
-   Count (done)
-   Strict bounds (done)
-   Negative indexes (done)
-   Multiple indexes (done)
-   Skipping empty fields (done)
-   Strict return (done)
    -   Doesn't work (done)
-   Strict range order (done)
-   Replace delimiters:
    -   --replace-range-delimiter (done)
    -   --simple-ranges: just use the same newline as selections (done)
-   Replace newline between selections --join (done)
    -   Placeholder to keep invalid selections (done)
-   --invert flag, to choose everything EXCEPT our indexes (done)
-   Change to run once over each line of the input (done)
    -   The legacy implementation can be called -w/--whole (done)
    -   Disabling it can be called -p/--per-line (this is the default) (done)
    -   With count, it should count per line unless -w is active (done)

## Core Feature Refinement

On reflection, entirely on accident this is essentially a more powerful version of cut. It makes sense to stylise it as a drop-in replacement, which would mean implementing some expected cut features.

### Input Modes

-   Add -z mode, splitting by \0 instead of \n
    -   Add option to not print the trailing newline in this mode. Might just add this feature in general
-   Add support for cut's weird `-d','` type syntax (I think it also accepts `-d,`)
-   Input error: If a specified --input file can't be read, `exit 2` for I/O error

### Selection Mode

-   Add feature for byte slicing -b/--byte (done)
-   Add feature for character slicing -c/--char (done)
-   Add --f/--field mode, which accepts input selections
-   Add support for comma-separated selections
-   Add feature to skip empty lines
-   Add byte-based field parsing
    -   Maybe with --delimiter-bytes or a separate mode

### Delimiter Mode

-   Add the ability to select the delimiter BEFORE each item rather than after. Wouldn't be relevant in `cut`, but is when you're dealing with regex!
-   Make --placeholder accept an optional value
    -   Should support string or hex in bytemode!
-   Field separation flags for --join
    -   @auto: follows existing logic, try after-previous, then try before-next, then space
    -   @after-previous: delimiter from after previous field
    -   @before-next: delimiter from before next field
    -   @empty-byte: inserts an empty byte
    -   @none: equivalent to ""
-   Add --join-ranges, which only applies within ranges
-   Add --join-selections, which only applies between discrete selections
-   Add --join-records, which applies between each record

### Documentation Overhaul

-   Build a proper website documenting the current app.
    -   It should explain each of the 'mode' selections
    -   It should include good explanations and examples for various features
    -   A beautiful front page would be really nice
    -   FINALLY we need a decent usecases page. With the `cut` comparison, we can distinctly focus on what we do better.

## Stretch Features

-   Drop --input OR replace the string with a file input (done)
-   Functions to turn off the strict modes (done)
-   --format that can auto-output json or csv for the user (dropped)
-   -o,--output for specifying output file (done)
-   Zero-indexing
-   --list to show you a list of each item with its index
-   Special `start` `end` or `first` `last` tags in ranges

## Documentation

-   Add a list of common usecases:
    -   echo $PATH | splitby -w -d ":"
    -   Stripping the headers from a csv
    -   Grabbing a particular piece of information from some output
    -   Getting specific columns from a file
        -   Stripping a value from a dataset
    -   Wordcount
    -   Auto-fill columns of CSV or TSV, when it is missing items
