import os  
import subprocess

def count_lines_of_code():
    """
    Counts the lines of code of all .rs files in the src directory
    """

    total_lines = 0

    for file_info in os.walk("src"):
        for file in file_info[2]:
            if not file.endswith(".rs"):
                continue

            path = file_info[0] + '\\' + file

            # Open the file in read mode, ignoring any encoding errors
            with open(path, 'r') as f:
                lines = sum(1 for _ in f)

                print(f"{path} : {lines}")

                # Concise way to count lines using a generator expression
                total_lines += lines

    # Print the total number of lines found
    print(f"\nTotal lines of code: {total_lines}")

if __name__ == "__main__":
    count_lines_of_code()