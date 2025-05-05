#!/bin/bash

delimiter=""   # default regex for whitespace
input=""
input_file=""
input_file_provided=0
join_string=$'\n'
range_delimiter=""
range_delimiter_provided=0
simple_ranges=0
count=0
skip_empty=0
placeholder=0
strict_return=0
strict_bounds=0
strict_range_order=1

show_help() {
    echo
    echo "Split a string by a delimiter and return a selection of the result."
    echo
    echo "Usage: splitby [options] -d <delimiter> index_or_range"
    echo
    echo "Options:"
    echo "  -d, --delimiter <regex>                 Specify the delimiter to use (required)"
    echo "  -i, --input <input_file>                Provide input file"
    echo "  -j, --join <string>                     Join selections with <string>"
    echo "      --replace-range-delimiter <string>  Replaces the delimiters within ranges"
    echo "      --simple-ranges                     Treat ranges as a list of selections"
    echo "      --no-simple-ranges                  Turn off simple ranges"
    echo "  -c, --count                             Return the number of results"
    echo "  -e, --skip-empty                        Skip empty fields"
    echo "  -E, --no-skip-empty                     Turn off skipping empty fields"
    echo "      --placeholder                       Preserves invalid selections in output"
    echo "  -s, --strict                            Shorthand for all strict features"
    echo "  -S, --no-strict                         Turn off all strict features"
    echo "      --strict-return                     Emit error if there is no usable result"
    echo "      --no-strict-return                  Turn off strict return"
    echo "      --strict-bounds                     Emit error if range is out of bounds"
    echo "      --no-strict-bounds                  Turn off strict bounds"
    echo "      --strict-range-order                Emit error if the start of a range is greater than the end (default: true)"
    echo "      --no-strict-range-order             Turn off strict range order"
    echo "  -h, --help                              Display this help message"
    echo "  -v, --version                           Show the current version"
    echo
    echo "Example:"
    echo "  echo \"this is a test\" | splitby -d ' ' 2            # Extract 2nd field"
    echo "  > is"
    echo "  echo \"this is a test \" | splitby -d ' ' -1          # Extract last field"
    echo "  > test"
    echo "  echo \"this is a test\" | splitby -d ' ' 1-3   # Extract fields from 1 to 3"
    echo "  > this is a"
    echo "  splitby -i test.txt -d ',' 2--1                # Extract fields from 2 to the last item"
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
            echo "1.0.0"
            exit 0
            ;;
        --delimiter=*)
            delimiter="${1#--delimiter=}"
            if [[ -z "$delimiter" ]]; then
                echo "Error: non-empty delimiter not currently supported" >&2
            fi
            shift
            ;;
        -d|--delimiter)
            delimiter="$2"
            shift 2
            ;;
        -i|--input)
            input_file="$2"
            input_file_provided=1
            shift 2
            ;;
        --join=*)
            join_string="${1#--join=}"
            shift
            ;;
        -j|--join)
            if [[ -z "${2+x}" ]]; then
                echo "Error: --join requires a value (use \"\" for empty string)" >&2
                exit 1
            fi
            join_string="$2"
            shift 2
            ;;
        --replace-range-delimiter=*)
            range_delimiter="${1#--replace-range-delimiter=}"
            range_delimiter_provided=1
            shift
            ;;
        --replace-range-delimiter)
            if [[ -z "${2+x}" ]]; then
                echo "Error: --replace-range-delimiter requires a value (use \"\" for empty string)" >&2
                exit 1
            fi
            range_delimiter="$2"
            range_delimiter_provided=1
            shift 2
            ;;
        --simple-ranges)
            simple_ranges=1
            shift
            ;;
        -c|--count)
            count=1
            shift
            ;;
            
        -e|--skip-empty)
            skip_empty=1
            shift
            ;;
        -E|--no-skip-empty)
            skip_empty=0
            shift
            ;;
        --placeholder)
            placeholder=1
            shift
            ;;
        -s|--strict)
            strict_bounds=1
            strict_return=1
            strict_range_order=1
            shift
            ;;
        -S|--no-strict)
            strict_bounds=0
            strict_return=0
            strict_range_order=0
            shift
            ;;
        --strict-return)
            strict_return=1
            shift
            ;;
        --no-strict-return)
            strict_return=0
            shift
            ;;
        --strict-bounds)
            strict_bounds=1
            shift
            ;;
        --no-strict-bounds)
            strict_bounds=0
            shift
            ;;
        --strict-range-order)
            strict_range_order=1
            shift
            ;;
        --no-strict-range-order)
            strict_range_order=0
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

# --- Check for input file ---
if [[ "$input_file_provided" -eq 1 ]]; then
    if [[ -z "$input_file" ]]; then
        echo "-i flag used but no input file provided." >&2
        exit 1
    fi
    input=$(cat $input_file)
else
    if [[ -t 0 ]]; then
        echo "No input provided. Use -i/--input or pipe data to stdin." >&2
        exit 1
    fi
    input=$(cat)
fi

# --- Check for empty input ---
if [[ -z "$input" ]]; then
    if [[  $input_file_provided -eq 1 ]]; then
        # The input file was empty, safe to just end here.
        exit 0
    fi
    
    # They didn't pipe anything
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

    my ($input, $regex_raw, $join_string, $simple_ranges, $range_delimiter, $range_delimiter_provided, $start_raw, $end_raw, $count, $strict_bounds, $strict_return, $strict_range_order, $skip_empty, $no_selection) = @ARGV;
    

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
        exit 111;
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
        exit 111;
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

    my $is_skipped = $skip_empty && $field eq "";

    if (!$is_skipped) {
        if ($field_index >= $start && $field_index <= $end) {
            push @output, $field;

            if ($field_index < $end && !$no_selection && !$simple_ranges) {
                push @output, $range_delimiter_provided ? $range_delimiter : $delim;
            }
        }
        $field_index++;  # only increment when not skipped
    }
}

    # Join the result into one string
    if ($no_selection || $simple_ranges) {
        my $result = join($join_string, @output);
        print $result;
    } else {
        my $result = join("", @output);
        print $result;
    }
'

result=""
skip_join=0
for ((i = 0; i < ${#starts[@]}; i++)); do
    start="${starts[i]}"
    end="${ends[i]}"
    
    out=$(perl -e "$perl_script" "$input" "$delimiter" "$join_string" "$simple_ranges" "$range_delimiter" "$range_delimiter_provided" "$start" "$end" "$count" "$strict_bounds" "$strict_return" "$strict_range_order" "$skip_empty" "$no_selection" 2>&1)
    code=$?
    
    # Error code
    if [[ $code -eq 111 ]]; then
        # Invalid selector, skip join
        skip_join=1
    elif [[ $code -ne 0 ]]; then
        echo "$out"
        exit $code
    fi
    
    # Adding selection joiner
    if [[ $i -ne 0 ]]; then
        if [[ $skip_join -eq 0 ]] || [[ $placeholder -eq 1 ]]; then
            result+="$join_string"
        fi
    fi
    skip_join=0
    
    result+="$out"
done

# Skip echo if it isn't going to say anything
if [[ ! -z $result ]]; then
    echo -e "$result"
fi
