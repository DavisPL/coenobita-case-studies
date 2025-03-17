#!/bin/bash

echo "COMPARING CRATES..."
echo "----------------------------------------"

# Function to compare a single crate
compare_crate() {
    local dir=$1
    local out=$2

    # Check if directory exists
    if [ ! -d "$dir" ]; then
        echo "[ERROR] Directory '$dir' not found"
        exit 1
    fi

    # Check if corresponding original directory exists
    if [ ! -d "originals/$dir" ]; then
        echo "[ERROR] No corresponding directory 'originals/$dir' found"
        exit 1
    fi

    # Run diff with exclusions and count differences
    diff_count=$(diff --suppress-common-lines \
                      --side-by-side  \
                      --exclude ".*" \
                      --exclude "Cargo.lock" \
                      --exclude "Cargo.toml" \
                      --exclude "*.log" \
                      --exclude "target" \
                      --recursive "walkdir" "originals/walkdir" | grep -v "^diff" | wc -l)
    
    # Print the number of differing lines
    printf "%-10s | %d\n" "$dir" $diff_count

    if [ "$out" -eq 1 ]; then
        # Also print the output
        diff --suppress-common-lines \
             --exclude ".*" \
             --exclude "Cargo.lock" \
             --exclude "Cargo.toml" \
             --exclude "*.log" \
             --exclude "target" \
             --recursive "walkdir" "originals/walkdir" | grep -v "^diff"
    fi

    return $diff_count
}

# Check if 'originals' directory exists
if [ ! -d "originals" ]; then
    echo "[ERROR] 'originals' directory not found"
    exit 1
fi

# If the crate argument is provided, only compare that one
if [ $# -eq 1 ]; then
    compare_crate "$1" 1
    exit 0
fi

# Otherwise, compare all crates
for crate in */; do
    dir=${crate%/}

    # Skip the 'originals' directory
    if [ "$dir" = "originals" ]; then
        continue
    fi

    compare_crate "$dir" 0
done

exit 0
