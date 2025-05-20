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

## Stretch Features

-   Drop --input OR replace the string with a file input (done)
-   Functions to turn off the strict modes (done)
-   --format that can auto-output json or csv for the user (dropped)
-   -o,--output for specifying output file
-   Zero-indexing
-   --list to show you a list of each item with its index

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
