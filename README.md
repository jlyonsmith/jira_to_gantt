# Jira CSV to Gantt Chart JSON Converter

This program converts Jira CSV exports to a format that can be ingested by the [gantt_chart]() project.

## Notes

Jira CSV export has many problems.

- There are extra non-CSV lines at the start and end of the output
- Dates are in a non ISO format
- The backlog sort order is missing
- There are bad UTF-8 characters in the output
- Quoting is inconsistent

Use `iconv -c -t utf-8 bad.csv > stripped.csv` to clean bad UTF-8 characters from export. See [iconv](https://www.shellhacks.com/linux-check-change-file-encoding/).

Use `xsv slice -s 3 -n -o bad.csv jira.csv` to remove the first 3 lines.
