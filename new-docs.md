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
--classic                 keep delimiters INSIDE ranges (original behaviour)
--simple-ranges           treat every range as list of discrete fields
--invert                  output everything except the selections
--count                   output counts instead of text
-z, --zero-terminated     read/write NUL-separated records
-w, --whole-string        process the entire input as one record
```

## Doc Sections

-   Quick Start
-   Flags
    -   Input flags
    -   Mode flags
    -   Delimiter flags
-   Cut comparison cheatsheet
-   FAQ
    -   E.g. starting delimiters causing trouble
