import os
import csv
import configparser
import csv
import pandas as pd


def count_lines_in_csv(directory: str, file: str) -> int:
    """
    Gets the number of lines in a csv file
    """
    csv_path = os.path.join(directory, file)
    if not os.path.isfile(csv_path):
        print(f"No 'actual.csv' found in {directory}")
        return

    with open(csv_path, "r") as csv_file:
        csv_reader = csv.reader(csv_file)
        line_count = sum(1 for _ in csv_reader)

    return line_count


def get_expected_and_actual_lines(directory: str) -> dict:
    """
    Gets the number of lines in actual and expected files
    """
    config = configparser.ConfigParser(allow_no_value=True)
    config.read(os.path.join(directory, "config.ini"))
    FILE_NAMES = config["file_names"]
    return {
        "Test Case": os.path.basename(directory),
        "Verilog Clock Cycles": count_lines_in_csv(directory, FILE_NAMES["actual"]),
        "Python Yield Iterations": count_lines_in_csv(
            directory, FILE_NAMES["expected"]
        ),
    }


# Get the directory where the script is located
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
STATS_CSV = os.path.join(SCRIPT_DIR, "stats.csv")

# Iterate over the subdirectories in the script directory
tests = []
for root, dirs, files in os.walk(SCRIPT_DIR):
    for directory in dirs:
        directory_path = os.path.join(root, directory)
        tests.append(get_expected_and_actual_lines(directory_path))

with open(STATS_CSV, mode="w") as stats_csv:
    writer = csv.DictWriter(stats_csv, fieldnames=tests[0].keys())
    writer.writeheader()
    writer.writerows(tests)

df = pd.read_csv(STATS_CSV)
with open(os.path.join(SCRIPT_DIR, "stats.md"), mode="w") as stats_md:
    df.to_markdown(buf=stats_md, index=False)
