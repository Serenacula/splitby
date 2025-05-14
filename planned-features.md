# Planned features

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
    -   Don't miss invert + count
-   --list to show you a list of each item with its index
-   -m/--multiline to run the script once over every line in the input
    -   Would this be good as on by default..?
    -   Multiline + count should have one count per line
    -   Could also be called -p/--per-line

# Post-Feature Refinement

-   Have a think about sensible defaults for each option
    -   Especially strict-range
-   Optimisation:
    -   Work out how to improve the looping. Maybe it can all be brought into a single loop?
    -   The loop inside perl should start at $start, not at 0
    -   Tbh I just don't really like the current structure. Might be better to bring all the looping code into perl

## Stretch Features

-   Drop --input OR replace the string with a file input (done)
-   --output for output file
-   --format that can auto-output json or csv for the user
-   Zero-indexing
-   Functions to turn off the strict modes (done)

## Documentation

-   Add a list of common usecases:
    -   echo $PATH | splitby -d ":"
    -   Stripping the headers from a csv
    -   Grabbing a particular piece of information from some output
    -   Getting specific columns from a file
        -   Stripping a value from a dataset
    -   Wordcount
