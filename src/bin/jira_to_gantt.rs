use colored::Colorize;
use core::fmt::Arguments;
use jira_to_gantt::{error, JiraToGanttLog, JiraToGanttTool};

struct JiraToGanttLogger;

impl JiraToGanttLogger {
    fn new() -> JiraToGanttLogger {
        JiraToGanttLogger {}
    }
}

impl JiraToGanttLog for JiraToGanttLogger {
    fn output(self: &Self, args: Arguments) {
        println!("{}", args);
    }
    fn warning(self: &Self, args: Arguments) {
        eprintln!("{}", format!("warning: {}", args).yellow());
    }
    fn error(self: &Self, args: Arguments) {
        eprintln!("{}", format!("error: {}", args).red());
    }
}

fn main() {
    let logger = JiraToGanttLogger::new();

    if let Err(error) = JiraToGanttTool::new(&logger).run(std::env::args_os()) {
        error!(logger, "{}", error);
        std::process::exit(1);
    }
}
