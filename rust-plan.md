# Rust Version Plan

-   Get the argument flags working (done)
    -   Done with clap
-   Selection parsing into range vectors
-   We want to choose a regex engine based on requirements. Rust's one is fast but simple, or there is a package we can use for fancy regex
-   Parallelisation for line or zero terminated input
    -   If we can check the input size before starting to see if it's necessary that would be cool

## Later features

-   See if we can do parallelisation with whole-string by chunking it
    -   Would only work with simple regex patterns, not fancy
