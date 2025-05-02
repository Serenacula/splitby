#!/bin/bash

delimiter=""   # default regex for whitespace
input=""
count=0
strict_bounds=0
strict_return=0
strict_range_order=0
skip_empty=0

show_help() {
    echo
    echo "Split a string by a delimiter and return a selection of the result."
    echo
    echo "Usage: splitby [options] -d <delimiter> index_or_range"
    echo
    echo "Options:"
    echo "  -d,   --delimiter <regex>     Specify the delimiter to use (required)"
    echo "  -i,   --input <input_string>  Provide input string directly"
    echo "  -c,   --count                 Return the number of results"
    echo "  -s,   --strict                Shorthand for all strict features"
    echo "  -sb,  --strict-bounds         Emit error if range is out of bounds (default: disabled)"
    echo "  -sr,  --strict-return         Emit error if there is no usable result"
    echo "  -sro, --strict-range-order    Emit error if the start of a range is greater than the end"
    echo "  -e,   --skip-empty            Skip empty fields"
    echo "  -h,   --help                  Display this help message"
    echo "  -v,   --version               Show the current version"
    echo
    echo "Example:"
    echo "  echo \"boo hoo\" | splitby -d ' ' 2            # Extract 2nd field"
    echo "  > hoo"
    echo "  echo \"boo hoo \" | splitby -d ' ' -1          # Negative values count backwards from the end"
    echo "  > hoo"
    echo "  echo \"this is a test\" | splitby -d ' ' 1-3   # Extract fields from 1 to 3"
    echo "  > this is a"
    echo "  splitby -i \"this,is,a,test\" -d ',' 2--1      # Extract fields from 2 to the last item"
    echo "  > is a test"
    echo
    exit 0
}

# --- Check for Perl dependency ---
required_version="5.10.0"

if ! command -v perl &> /dev/null; then
    echo "Error: Perl is required but not installed." >&2
    exit 1
fi

installed_version=$(perl -e 'print $^V')

if [[ "$installed_version" < "$required_version" ]]; then
    echo "Error: Perl version $required_version or higher is required. Installed version is $installed_version." >&2
    exit 1
fi


# --- Parse options ---
selections=()  # To store all range/index inputs

while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            show_help
            ;;
        -v|--version)
            echo "1.1.0"
            exit 0
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
        -s|--strict)
            strict_bounds=1
            strict_return=1
            strict_range_order=1
            shift
            ;;
        -sb|--strict-bounds)
            strict_bounds=1
            shift
            ;;
        -sr|--strict-return)
            strict_return=1
            shift
            ;;
        -sro|--strict-range-order)
            strict_range_order=1
            shift
            ;;
        -e|--skip-empty)
            skip_empty=1
            shift
            ;;
        --)
            shift
            break
            ;;
        *)
            # Could be a negative number like -3 or a malformed flag
            if [[ "$1" =~ ^-?[0-9]+(--?[0-9]+)?$ ]]; then
                selections+=("$1")
                shift
            else
                echo "Unknown option or invalid input: $1, use -h or --help to see usage."
                exit 1
            fi
            ;;
    esac
done

# --- Ensure delimiter is provided ---
if [[ -z "$delimiter" ]]; then
    echo "Delimiter is required. Use -d or --delimiter to set one." >&2
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

# --- Parse range ---
# If no selections provided, select everything
no_selection=0
[[ ${#selections[@]} -eq 0 ]] && no_selection=1 && selections+=("")

# For each selection, validate format and prepare ranges
starts=()
ends=()

for selection in "${selections[@]}"; do
    if [[ "$selection" =~ ^(-?[0-9]+)$ ]]; then
        # Single index
        starts+=("${BASH_REMATCH[1]}")
        ends+=("${BASH_REMATCH[1]}")
    elif [[ "$selection" =~ ^(-?[0-9]+)-(-?[0-9]+)$ ]]; then
        start="${BASH_REMATCH[1]}"
        end="${BASH_REMATCH[2]}"

        # Disallow wrapping if both are same sign
        if [[ "$start" =~ ^- && "$end" =~ ^- ]] || ([[ "$start" =~ ^[0-9] ]] && [[ "$end" =~ ^[0-9] ]]); then
            if (( end < start )) && [[ $strict_range_order -eq 1 ]]; then
                echo "Error: end index ($end) is less than start index ($start) in selection '$selection'" >&2
                exit 1
            fi
        fi

        starts+=("$start")
        ends+=("$end")
    elif [[ -z "$selection" ]]; then
        # Empty selection means full range
        starts+=("1")
        ends+=("-1")
    else
        echo "Invalid selection syntax: '$selection'" >&2
        exit 1
    fi
done

# --- Run Perl split ---
perl_script='
    use strict;
    use warnings;

    my ($input, $regex_raw, $start_raw, $end_raw, $count, $strict_bounds, $strict_return, $strict_range_order, $skip_empty, $no_selection) = @ARGV;
    

    my $regex = eval { qr/$regex_raw/ };
    if ($@) {
        die "Invalid delimiter regex: $regex_raw\n";
    }
    
    my @data_parts_raw = split /(?:$regex)/, $input, -1;
    my $num_data_parts_raw = scalar @data_parts_raw;
    
    my @data_parts = grep { !$skip_empty || ($_ ne "") } @data_parts_raw;
    my $num_data_parts = scalar @data_parts;
    
    if ($num_data_parts_raw == 1) {
        if ($count) {
            print "0\n";
            exit 0;
        }
        if ($strict_return) {
            die "Strict empty check failed: No valid fields available\n";
        }
        exit 0;
    }
    
    if ($count) {
        print "$num_data_parts\n";
        exit 0;
    }

    if ($num_data_parts == 0) {
        if ($strict_return) {
            die "Strict empty check failed: No valid fields available\n";
        }
        exit 0;
    }

    # Convert start index (can be negative)
    my $start;
    if ($start_raw =~ /^-[0-9]+$/) {
        $start = $num_data_parts + $start_raw;  # e.g. -1 means last element
    } elsif ($start_raw =~ /^[0-9]+$/) {
        $start = $start_raw - 1;  # Convert to 0-indexed
    } else {
        die "Start index must be an integer.\n";
    }

    # Convert end index (can be empty or negative)
    my $end;
    if ($end_raw =~ /^-[0-9]+$/) {
        $end = $num_data_parts + $end_raw;
    } elsif ($end_raw =~ /^[0-9]+$/) {
        $end = $end_raw - 1;
    } else {
        die "End index must be an integer.\n";
    }
    
    

    # Invalid range
    if ($end < $start && !$no_selection) {
        if ($strict_range_order) {
            die "End index ($end_raw) is less than start index ($start_raw) in selection $start_raw-$end_raw\n";
        }
        exit 0;
    }

    # Strict bounds handling (optional)
    if ($strict_bounds) {
        if ($start < 0 || $start > $#data_parts) {
            if ($start_raw == $end_raw) {
                die "Index ($start_raw) out of bounds. Must be between 1 and " . ($#data_parts + 1) . "\n";
            }
            die "Start index ($start_raw) out of bounds. Must be between 1 and " . ($#data_parts + 1) . "\n";
        }
        if ($end < 0 || $end > $#data_parts) {
            die "End index ($end_raw) out of bounds. Must be between 1 and " . ($#data_parts + 1) . "\n";
        }
    }

    # Gracefully ignore out-of-bounds indices if not strict bounds
    if ($start > $#data_parts || $end < 0) {
        # print "\n";
        exit 0;
    }
    if ($start < 0) {
        $start = 0;
    }
    if ($end > $#data_parts) {
        $end = $#data_parts;
    }
    

    # Combine fields within range, preserving delimiters
    my @output;
    my $field_index = 0;
    my @parts = split /($regex)/, $input;  # Split with capturing groups to preserve delimiters
    for (my $i = 0; $i < @parts; $i += 2) {
        my $field = $parts[$i];
        my $delim = $parts[$i + 1] // "";
        
        if ($field_index >= $start && $field_index <= $end) {
            push @output, $field;
            if ($field_index < $end && !$no_selection) {
                push @output, $delim;  # Always keep the delimiter
            }
        }
        
        $field_index++ unless $skip_empty && $field eq "";
    }

    # Join the result into one string
    if ($no_selection) {
        my $result = join("\n", @output);
        print $result;
    } else {
        my $result = join("", @output);
        print $result;
    }
'

result=""
for ((i = 0; i < ${#starts[@]}; i++)); do
    if [[ $i -ne 0 ]]; then
        result+=$'\n'
    fi
    
    start="${starts[i]}"
    end="${ends[i]}"
    
    out=$(perl -e "$perl_script" "$input" "$delimiter" "$start" "$end" "$count" "$strict_bounds" "$strict_return" "$strict_range_order" "$skip_empty" "$no_selection" 2>&1)
    code=$?
    
    # Error code
    if [[ $code -ne 0 ]]; then
        echo "$out"
        exit $code
    fi
    
    result+="$out"
done

# Skip echo if it isn't going to say anything
if [[ ! -z $result ]]; then
    echo -e "$result"
fi
