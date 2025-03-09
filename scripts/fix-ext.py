# Fix image file extensions in a directory
# Usage: python fix-ext.py your_directory_path

import os
import imghdr
import sys


def correct_extension(directory):
    fixed = 0
    correct = 0
    unknown = 0

    for filename in os.listdir(directory):
        filepath = os.path.join(directory, filename)
        if os.path.isfile(filepath):
            file_type = imghdr.what(filepath)
            if file_type:
                new_filename = os.path.splitext(filename)[0] + "." + file_type
                new_filepath = os.path.join(directory, new_filename)
                if filename != new_filename:
                    os.rename(filepath, new_filepath)  # Comment to dry-run
                    print(f"Renamed {filename} to {new_filename}")
                    fixed += 1
                else:
                    correct += 1
            else:
                print(f"Unknown file type: {filename}")
                unknown += 1

    print("Summary:")
    print(f"Fixed: {fixed}")
    print(f"Correct: {correct}")
    print(f"Unknown: {unknown}")

if __name__ == "__main__":
    # Replace 'your_directory_path' with the path to your image directory
    target = sys.argv[1]
    correct_extension(target)
