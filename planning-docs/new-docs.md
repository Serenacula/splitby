# New Docs

Just a few notes on how the new docs should be arranged. Research into other good docs should also be done.

## Script Help

## Suggestion for new help text, once cut parity achieved

```
splitby -d REGEX [selection…]       # field mode (default)
splitby --bytes [selection…]        # byte offsets
splitby --chars [selection…]        # character offsets

Selections: 1 4 6-10  -3--1  etc.  Negative counts from end.

# Common flags
-j, --join STR            join string (default: original delimiter in field mode)
--invert                  output everything except the selections
--count                   output counts instead of text
-z, --zero-terminated     read/write NUL-separated records
-w, --whole-string        process the entire input as one record
```

## Doc Sections

-   Quick Start
-   Modes
    -   Input modes
        -   Mode: Per line (default)
        -   Mode: Whole string
        -   Mode: Zero-terminated
    -   Selection modes
        -   Mode: Fields (default)
        -   Mode: Characters
        -   Mode: Bytes
        -   Flag: Invert, reverse the selection
        -   Flag: Skip-empty, ignore empty fields
    -   Flag: Join, changes the delimiter used for connecting
-   Flags
    -   Input flags
    -   Selection flags
    -   Delimiter mode flags
        -   Cut mode: delimiters are kept between selections, which is how original cut works
        -   Classic mode: delimiters are dropped between selections, which is how mine worked
        -   Simple Ranges: DROPPED! Since it can already be achieved alternatively
        -   Join: changes the delimiter used for connecting fields
-   Cut comparison cheatsheet
    -   Differences in syntax!
-   FAQ
    -   E.g. starting delimiters causing trouble
