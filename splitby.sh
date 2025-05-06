#!/bin/bash

# Core features
delimiter=""   # default regex for whitespace
input=""
input_file=""
input_file_provided=0
selections=()  # To store all range/index inputs

# Options
join_string=$'\n'
simple_ranges=0
range_delimiter=""
range_delimiter_provided=0
count=0
skip_empty=0
placeholder=0

# Strict
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

# --- Run Perl split ---
perl_script='
    use strict;
    use warnings;

    my (
        $input, 
        $delimiter,
        $join_string, 
        $simple_ranges, 
        $range_delimiter, 
        $range_delimiter_provided, 
        $count, 
        $skip_empty, 
        $placeholder,
        $strict_return, 
        $strict_bounds, 
        $strict_range_order, 
        @selections
    ) = @ARGV;

    my $no_selection = 0;
    if (!@selections || (@selections == 1 && $selections[0] eq "")) {
        $no_selection = 1;
    }

    my $regex = eval { qr/$delimiter/ };
    if ($@) {
        die "Invalid delimiter regex: $delimiter\n";
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

    # Parsing selections into real values
    
    sub resolve_index {
        my ($raw, $total_fields) = @_;
        return $raw =~ /^-/ ? $total_fields + $raw : $raw - 1;
    }

    my @starts;
    my @ends;

    foreach my $selection (@selections) {
        if ($selection eq "") {
            push @starts, 0;
            push @ends, $#data_parts;
            next;
        }

        # Single index
        if ($selection =~ /^(-?\d+)$/) {
            my $index = resolve_index($1, scalar(@data_parts));

            if ($strict_bounds) {
                if ($index < 0 || $index > $#data_parts) {
                    die "Index ($1) out of bounds. Must be between 1 and " . ($#data_parts + 1) . "\n";
                }
            } elsif ($index < 0 || $index > $#data_parts) {
                # Push empty placeholder if enabled
                if ($placeholder) {
                    push @starts, -1;
                    push @ends, -1;
                }
                next;
            }

            push @starts, $index;
            push @ends, $index;
            next;
        }

        # Range
        if ($selection =~ /^(-?\d+)-(-?\d+)$/) {
            my ($start_raw, $end_raw) = ($1, $2);
            my $start = resolve_index($start_raw, scalar(@data_parts));
            my $end   = resolve_index($end_raw, scalar(@data_parts));

            if ($strict_range_order && $end < $start) {
                die "End index ($end_raw) is less than start index ($start_raw)\n";
            }

            if ($strict_bounds) {
                if ($start < 0 || $start > $#data_parts) {
                    die "Start index ($start_raw) out of bounds. Must be between 1 and " . ($#data_parts + 1) . "\n";
                }
                if ($end < 0 || $end > $#data_parts) {
                    die "End index ($end_raw) out of bounds. Must be between 1 and " . ($#data_parts + 1) . "\n";
                }
            } else {
                $start = 0 if $start < 0;
                $end = $#data_parts if $end > $#data_parts;

                if ($start > $#data_parts || $end < 0) {
                    if ($placeholder) {
                        push @starts, -1;
                        push @ends, -1;
                    }
                    next;
                }
            }

            push @starts, $start;
            push @ends, $end;
            next;
        }


        die "Invalid selection syntax: \"$selection\"\n";
    }

    my @parts = split /($regex)/, $input;
    my @output_selections = ();

    for (my $selection_index = 0; $selection_index < @starts; $selection_index++) {
        my $start = $starts[$selection_index];
        my $end   = $ends[$selection_index];

        # Placeholder case
        if ($start == -1 && $end == -1) {
            push @output_selections, "";
            next;
        }

        my @selection_output = ();
        my $field_index = 0;

        for (my $i = 0; $i < @parts; $i += 2) {
            my $field = $parts[$i];
            my $delim = $parts[$i + 1] // "";

            my $is_skipped = $skip_empty && $field eq "";
            next if $is_skipped;

            if ($field_index >= $start && $field_index <= $end) {
                push @selection_output, $field;

                if ($field_index < $end && !$no_selection && !$simple_ranges) {
                    push @selection_output, $range_delimiter_provided ? $range_delimiter : $delim;
                }
            }

            $field_index++;
        }

        my $joined_selection = ($no_selection || $simple_ranges)
            ? join($join_string, @selection_output)
            : join("", @selection_output);

        push @output_selections, $joined_selection;
    }
    
    if ($strict_return && !grep { $_ ne "" } @output_selections) {
        die "Strict return check failed: No valid selections were output\n";
    }
    
    my $result = join($join_string, @output_selections);
    print $result;
'

if [[ ${#selections[@]} -eq 0 ]]; then
    selections+=("")
fi

result=$(perl -e "$perl_script" \
    "$input" \
    "$delimiter" \
    "$join_string" \
    "$simple_ranges" \
    "$range_delimiter" \
    "$range_delimiter_provided" \
    "$count" \
    "$skip_empty" \
    "$placeholder" \
    "$strict_return" \
    "$strict_bounds" \
    "$strict_range_order" \
    "${selections[@]}"\
    2>&1)
code=$?

if [[ $code -ne 0 ]]; then
    echo "$result" >&2
    exit $code
fi

if [[ -n $result ]]; then
    echo -e "$result"
fi

