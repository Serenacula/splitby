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

-   Add feature for byte slicing -b/--byte
-   Add feature for character slicing -c/--char
-   Add --f/--field mode, which accepts input selections
-   Add support for comma-separated selections
-   Add feature to skip empty lines (--only-delimited or --skip-delimited)

### Delimiter Mode

-   Add 'cut' mode as the default - this keeps delimiters between selections.
    -   --join should replace between ranges in this mode
    -   Add the ability to select the delimiter BEFORE each item rather than after. Wouldn't be relevant in `cut`, but is when you're dealing with regex!
-   Have a think about whether --simple-ranges is worth keeping as a feature.
-   Map --output-delimiter to --join

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
-   -o,--output for specifying output file
-   Zero-indexing
-   --list to show you a list of each item with its index
-   Special `start` `end` or `first` `last` tags in ranges

## Post-Feature Refinement

-   Have a think about sensible defaults for each option
    -   Especially strict-range
-   Optimisation:
    -   Work out how to improve the looping. Maybe it can all be brought into a single loop?
    -   The loop inside perl should start at $start, not at 0
    -   Tbh I just don't really like the current structure. Might be better to bring all the looping code into perl (done)
-   Rebuild:
    -   Once the core structure is in place and I know how I want it designed, it will be rebuilt in rust. It should prove a good way to learn the language, while also improving the speed.

## Documentation

-   Add a list of common usecases:
    -   echo $PATH | splitby -w -d ":"
    -   Stripping the headers from a csv
    -   Grabbing a particular piece of information from some output
    -   Getting specific columns from a file
        -   Stripping a value from a dataset
    -   Wordcount
    -   Auto-fill columns of CSV or TSV, when it is missing items
