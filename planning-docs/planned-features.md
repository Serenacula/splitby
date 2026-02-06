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

### General

-   Add config file to control defaults
-   Optimise multithreading performance (done)
    -   Note: Done via record batching, we're now performant with cut in almost all cases and beat it at larger numbers of fields
-   I think I want to move to a hand-rolled parser instead of clap. Mostly because I want to be able to put the delimiter before other flags, and just have a bit more arbitrary ordering of flags. Clap is a bit too restrictive for this (done)

### Input Modes

-   Add -z mode, splitting by \0 instead of \n (done)
    -   Add option to not print the trailing newline in this mode. Might just add this feature in general (dropped)
        -   ~~DROPPED, it caused more problems than it helped. Instead -z ends in \0, -p ends in \n, -w keeps what it had~~
        -   RENEW, whole-string should print final newline if outputting to terminal (done)
        -   RENEW, add an option to control the trailing newline
-   Add support for cut's weird `-d','` type syntax (I think it also accepts `-d,`) ~~(dropped)~~ (done)
    -   ~~DROPPED, caused too many problems and inconsistent cli syntax~~
    -   Done, as part of rolling a custom cli parser
-   Input error: If a specified --input file can't be read, `exit 2` for I/O error (done)
-   Code point input mode
    -   Cut parity, since --characters doesn't do this anymore

### Selection Mode

-   Add feature for byte slicing -b/--byte (done)
-   Add feature for character slicing -c/--char (done)
-   Add --f/--fields mode, which accepts input selections (done)
-   Add support for comma-separated selections (done)
-   Add feature to skip empty lines
    -   Should it skip empty inputs or empty outputs?
-   Add feature to skip lines with no delimiter
    -   Cut parity, -s/--only-delimited
-   Add byte-based field parsing
    -   Maybe with --delimiter-bytes or a separate mode

### Output

-   Add the ability to select the delimiter BEFORE each item rather than after. Wouldn't be relevant in `cut`, but is when you're dealing with regex! (done)
-   Make --placeholder accept a value (done)
    -   Should support string or hex in bytemode! (done)
    -   Needs a fallback delimiter for this, maybe last seen? (done)
-   Field separation flags for --join (done)
    -   @auto: follows existing logic, try after-previous, then try before-next, then space (done)
    -   @after-previous: delimiter from after previous field (done)
    -   @before-next: delimiter from before next field (done)
    -   @none: equivalent to "" (done)
-   Add --join-ranges, which only applies within ranges
-   Add --join-selections, which only applies between discrete selections
-   Add --join-records, which applies between each record
-   Add --align, auto aligns the fields with spaces in between (done)
    -   Want to be able to define how the alignment is handled. Options are: (done)
        -   Align text left, padding before delimiter, e.g.: `a   |b   |c` (done)
        -   Align text left, padding after delimiter, e.g.: `a,   b,   c` (done)
        -   Align text right, padding before text, e.g.: `   a,   b,   c` (done)

### Documentation

-   Build a proper website documenting the current app. (in progress)
    -   Make the theme look nice
    -   Go through and refine the docs
    -   Add a page comparing splitby with cut, and what's changed

## Stretch Features

-   Drop --input OR replace the string with a file input (done)
-   Functions to turn off the strict modes (done)
-   --format that can auto-output json or csv for the user (dropped)
-   -o,--output for specifying output file (done)
-   Zero-indexing
-   --list to show you a list of each item with its index
-   Special `start` `end` or `first` `last` tags in ranges (done)

## Documentation

-   Add a list of common usecases:
    -   echo $PATH | splitby -w -d ":"
    -   Stripping the headers from a csv
    -   Grabbing a particular piece of information from some output
    -   Getting specific columns from a file
        -   Stripping a value from a dataset
    -   Wordcount
    -   Auto-fill columns of CSV or TSV, when it is missing items
