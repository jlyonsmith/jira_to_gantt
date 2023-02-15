# Jira CSV to Gantt Chart JSON Converter

This program converts Jira CSV exports to a format that can be ingested by the [gantt_chart](https://crates.io/crates/gantt_chart) tool.

Install with `cargo install jira_to_gantt`.  Run with `jira-to-gantt`.

## Notes

The tool uses the following Jira fields:

- *Issue key* - Gives a short description of the item
- *Status* - Used to mark the task open or close on the chart
- *Assignee* - Used to group the tasks
- *Original Estimate* - Task duration
- *Created* (optional) - Used if no *Starts On* date is provided on the command line

Jira CSV export has numerous problems and inconsistencies which the tool handles, including:

- There are extra non-CSV format lines at the start and end of the output
- Dates are in a non ISO format
- Any backlog sort order is not honored
- There are bad UTF-8 characters in the output
- Quoting is inconsistent

You can use `iconv -c -t utf-8 bad.csv > stripped.csv` to clean bad UTF-8 characters from export. See [iconv](https://www.shellhacks.com/linux-check-change-file-encoding/). *The tool  does this automatically.*

You can use `xsv slice -s 3 -n -o bad.csv jira.csv` to remove the first 3 lines. *Again, the tool does this automatically.*

The tool uses structures from the [gantt_chart](https://crates.io/crates/gantt_chart) crate to ensure compatability of the [JSON5](http://json5.org) output.
