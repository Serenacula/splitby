#!/bin/bash

delimiter=""   # default regex for whitespace
input=""
index=""
count=0
strict_bounds=0

show_help() {
    echo
    echo "Split a string by a delimiter and return a selection of the result."
    echo
    echo "Usage: $0 [options] -d <delimiter> index_or_range"
    echo
    echo "Options:"
    echo "  -d, --delimiter <regex>     Set custom delimiter (required)"
    echo "  -i, --input <input_string>  Provide input string directly"
    echo "  -c, --count                 Return the number of results"
    echo "  -s, --strict-bounds         Fail if range is out of bounds (default: disabled)"
    echo "  --help                      Display this help message"
    echo
    echo "Example:"
    echo "  echo \"this is a test\" | $0 1-2   # Extract fields from 1 to 4"
    echo "    output:  this is"
    echo "  $0 -i \"this is a test\" 2-        # Extract fields from 2 to 3"
    echo "    output:  is a test"
    echo
    exit 0
}

# --- Parse options ---
while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            show_help
            ;;
        -d|--delimiter)
            delimiter="$2"
            shift 2
            ;;
        -i|--input)
            input="$2"
            shift 2
            ;;
        -c|--count)
            count=1
            shift
            ;;
        -s|--strict-bounds)
            strict_bounds=1
            shift
            ;;
        --)
            shift
            break
            ;;
        -*)
            if [[ "$1" =~ ^-[0-9]*$ ]]; then
                index="$1"  # Keep full -N (we’ll handle it in range parsing)
                shift
            else
                echo "Unknown option: $1"
                exit 1
            fi
            ;;
        *)
            index="$1"
            shift
            ;;
    esac
done


# --- Ensure index provided ---
if [[ -z "$index" ]] && [[ "$count" -eq 0 ]]; then
    echo "Usage: $0 [options] -d <delimiter> index_or_range" >&2
    exit 1
fi

# --- Read from stdin if no input string provided ---
if [[ -z "$input" ]]; then
    if [[ -t 0 ]]; then
        echo "No input provided. Use -i/--input or pipe data to stdin." >&2
        exit 1
    fi
    input=$(cat)
fi

# --- Check for empty input ---
if [[ -z "$input" ]]; then
    echo "No input provided. Use -i/--input or pipe data to stdin." >&2
    exit 1
fi

# --- Ensure delimiter is provided ---
if [[ -z "$delimiter" ]]; then
    echo "Error: Delimiter is required. Use -d or --delimiter to set one." >&2
    exit 1
fi

# --- Parse range ---
if [[ "$index" =~ ^([0-9]*)-([0-9]*)$ ]]; then
    start="${BASH_REMATCH[1]}"
    end="${BASH_REMATCH[2]}"

    [[ -z "$start" ]] && start=1   # "-4" → from 1
    # Leave end blank if "2-" to signal open-ended range
else
    # Single number
    start="$index"
    end="$index"
fi

# --- Run Perl split ---
perl -e '
    use strict;
    use warnings;

    my ($input, $regex_raw, $start_raw, $end_raw, $count, $strict_bounds) = @ARGV;

    my $regex = eval { qr/$regex_raw/ };
    if ($@) {
        die "Invalid delimiter regex: $regex_raw\n";
    }
    
    if ($count) {
        my @data_parts = split /(?:$regex)/, $input;
        @data_parts = grep { $_ ne "" } @data_parts;
        
        print "$#data_parts\n";
        exit 0;
    }

    my @parts = split /($regex)/, $input;  # Split with capturing groups to preserve delimiters
    

    die "Start index is required.\n" unless defined $start_raw && $start_raw =~ /^\d+$/;
    my $start = $start_raw - 1;  # Convert to zero-indexed

    my $end;
    if (defined $end_raw && $end_raw ne "") {
        die "End index must be a number.\n" unless $end_raw =~ /^\d+$/;
        $end = $end_raw - 1;
    }
    

    # Strict bounds handling (optional)
    if ($strict_bounds) {
        my @data_parts = split /(?:$regex)/, $input;
        @data_parts = grep { $_ ne "" } @data_parts;
        
        if ($start < 0 || $start > $#data_parts) {
            die "Start index ($start_raw) out of bounds. Must be between 1 and " . ($#data_parts + 1) . "\n";
        }
        if ($end < 0 || $end > $#data_parts) {
            die "End index ($end_raw) out of bounds. Must be between 1 and " . ($#data_parts + 1) . "\n";
        }
        if ($end < $start) {
            die "End index ($end_raw) cannot be less than start index ($start_raw)\n";
        }
    }

    # Gracefully ignore out-of-bounds indices if not strict bounds
    if ($start > $#parts || $end < 0) {
        print "";
        exit 0;
    }
    if ($start < 0) {
        $start = 0
    }

    # Combine fields within range, preserving delimiters
    my @output;
    my $field_index = 0;
    for (my $i = 0; $i < @parts; $i += 2) {
        my $field = $parts[$i];
        my $delim = $parts[$i + 1] // "";

        if ($field_index >= $start && $field_index <= $end) {
            push @output, $field;
            if ($field_index < $end) {
                push @output, $delim;  # Always keep the delimiter
            }
        }
        $field_index++;
    }

    # Join the result into one string
    my $result = join("", @output);

    # Only add a newline at the end if the input doesnt already have one
    print $result;
    print "\n" unless $result =~ /\n\z/;
' "$input" "$delimiter" "$start" "$end" "$count" "$strict_bounds"
