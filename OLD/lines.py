import os
import sys
from pathlib import Path

def count_rust_lines(root_dir):
    """
    Count lines of Rust code in a project, excluding tests and examples.
    
    Args:
        root_dir (str): Root directory of the Rust project
        
    Returns:
        tuple: (total_lines, number_of_files, dict of files and their line counts)
    """
    total_lines = 0
    file_count = 0
    file_lines = {}
    
    # Convert to Path object for better path handling
    root_path = Path(root_dir)
    
    if not root_path.exists():
        print(f"Error: Directory '{root_dir}' does not exist.")
        sys.exit(1)
    
    # Walk through all directories
    for path in root_path.rglob('*.rs'):
        # Convert to string for easier pattern matching
        path_str = str(path)
        
        # Skip tests and examples
        if ('/tests/' in path_str or 
            'examples/' in path_str or 
            path_str.endswith('_test.rs') or
            path_str.endswith('.test.rs')):
            continue
            
        try:
            with open(path, 'r', encoding='utf-8') as f:
                # Read lines and filter out empty ones
                lines = [line.strip() for line in f.readlines()]
                non_empty_lines = [line for line in lines if line and not line.startswith('//')]
                line_count = len(non_empty_lines)
                
                # Update counters
                total_lines += line_count
                file_count += 1
                
                # Store individual file statistics
                relative_path = str(path.relative_to(root_path))
                file_lines[relative_path] = line_count
                
        except Exception as e:
            print(f"Error processing {path}: {str(e)}")
            
    return total_lines, file_count, file_lines

def main():
    # Get project directory from command line argument or use current directory
    if len(sys.argv) > 1:
        project_dir = sys.argv[1]
    else:
        project_dir = os.getcwd()
    
    total, files, file_stats = count_rust_lines(project_dir)
    
    # Print summary
    print("\nRust Project Line Count Summary")
    print("-" * 30)
    print(f"Total lines of code: {total}")
    print(f"Number of Rust files: {files}")
    
    # Print detailed breakdown
    print("\nBreakdown by file:")
    print("-" * 30)
    for file_path, lines in sorted(file_stats.items(), key=lambda x: x[1], reverse=True):
        print(f"{file_path}: {lines} lines")

if __name__ == "__main__":
    main()
