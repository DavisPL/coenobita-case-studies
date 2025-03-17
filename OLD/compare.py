import os
import hashlib
from pathlib import Path
from typing import Dict, Set, Tuple, List

# Default ignore patterns
DEFAULT_IGNORES = {
    '.DS_Store',
    '.git',
    '.github',
    'Cargo.toml',
    'Cargo.lock',
    'target',
    # '.cargo',
    '.vscode',
    'coenobita.log'
}

def should_ignore(path: Path, ignore_patterns: Set[str]) -> bool:
    """
    Check if a path should be ignored based on ignore patterns.
    Checks both file names and directory names in the path.
    """
    # Check if any part of the path matches ignore patterns
    return any(part in ignore_patterns for part in path.parts)

def get_file_hash(filepath: Path) -> str:
    """Calculate MD5 hash of a file."""
    hash_md5 = hashlib.md5()
    with open(filepath, "rb") as f:
        for chunk in iter(lambda: f.read(4096), b""):
            hash_md5.update(chunk)
    return hash_md5.hexdigest()

def get_directory_contents(directory: Path, ignore_patterns: Set[str]) -> Dict[str, str]:
    """
    Get a dictionary of relative file paths and their hashes for a directory,
    excluding ignored files and directories.
    """
    contents = {}
    for filepath in directory.rglob("*"):
        # Convert to relative path for ignore checking
        relative_path = filepath.relative_to(directory)
        
        # Skip if path should be ignored
        if should_ignore(relative_path, ignore_patterns):
            continue
            
        if filepath.is_file():
            contents[str(relative_path)] = get_file_hash(filepath)
    return contents

def compare_directories(
    dir1: str, 
    dir2: str, 
    ignore_patterns: Set[str] = DEFAULT_IGNORES
) -> Tuple[Set[str], Set[str], Set[str]]:
    """
    Compare two directories and return sets of:
    - Files only in dir1
    - Files only in dir2
    - Files that exist in both but have different contents
    
    Ignores files and directories matching the ignore patterns.
    """
    path1, path2 = Path(dir1), Path(dir2)
    
    if not path1.exists() or not path2.exists():
        raise ValueError("One or both directories do not exist")
        
    contents1 = get_directory_contents(path1, ignore_patterns)
    contents2 = get_directory_contents(path2, ignore_patterns)
    
    files1 = set(contents1.keys())
    files2 = set(contents2.keys())
    
    only_in_dir1 = files1 - files2
    only_in_dir2 = files2 - files1
    
    common_files = files1 & files2
    different_content = {
        f for f in common_files 
        if contents1[f] != contents2[f]
    }
    
    return only_in_dir1, only_in_dir2, different_content

def main():
    import sys
    
    if len(sys.argv) != 3:
        print("Usage: python script.py <directory1> <directory2>")
        sys.exit(1)
    
    dir1, dir2 = sys.argv[1], sys.argv[2]
    
    try:
        # Using default ignore patterns
        only_in_dir1, only_in_dir2, different_content = compare_directories(
            dir1, 
            dir2
        )
        
        if not (only_in_dir1 or only_in_dir2 or different_content):
            print("\nNo differences found (excluding ignored files/directories).")
            return
        
        if only_in_dir1:
            print("\nFiles only in first directory:")
            for f in sorted(only_in_dir1):
                print(f"  {f}")
            
        if only_in_dir2:
            print("\nFiles only in second directory:")
            for f in sorted(only_in_dir2):
                print(f"  {f}")
            
        if different_content:
            print("\nFiles with different content:")
            for f in sorted(different_content):
                print(f"  {f}")
            
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()