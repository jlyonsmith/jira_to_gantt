/// Generate a Gantt chart
use chrono::NaiveDate;
use clap::Parser;
use core::fmt::Arguments;
use csv::{self, ByteRecord, StringRecord};
use gantt_chart::{ChartData, ItemData};
use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::{self, Error as IoError, Read, Write};
use std::path::PathBuf;

mod log_macros;

const JIRA_DAY_IN_SECONDS: f32 = 8.0 * 60.0 * 60.0;

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Cli {
    /// Specify the JSON data file
    #[clap(value_name = "INPUT_FILE")]
    input_file: Option<PathBuf>,

    #[clap(value_name = "OUTPUT_FILE")]
    output_file: Option<PathBuf>,
}

impl Cli {
    fn get_output(&self) -> Result<Box<dyn Write>, IoError> {
        match self.output_file {
            Some(ref path) => File::open(path).map(|f| Box::new(f) as Box<dyn Write>),
            None => Ok(Box::new(io::stdout())),
        }
    }

    fn get_input(&self) -> Result<Box<dyn Read>, IoError> {
        match self.input_file {
            Some(ref path) => File::open(path).map(|f| Box::new(f) as Box<dyn Read>),
            None => Ok(Box::new(io::stdin())),
        }
    }
}

pub trait JiraToGanttLog {
    fn output(self: &Self, args: Arguments);
    fn warning(self: &Self, args: Arguments);
    fn error(self: &Self, args: Arguments);
}

pub struct JiraToGanttTool<'a> {
    log: &'a dyn JiraToGanttLog,
}

#[derive(Deserialize, Debug)]
struct JiraRecord {
    #[serde(rename = "Issue key")]
    key: String,
    #[serde(rename = "Summary")]
    #[allow(dead_code)]
    summary: String,
    #[serde(rename = "Status")]
    #[allow(dead_code)]
    status: String,
    #[serde(rename = "Assignee")]
    assignee: String,
    #[serde(rename = "Original Estimate", default)]
    original_estimate: Option<u32>,
    #[serde(rename = "Created")]
    created: String,
}

impl<'a> JiraToGanttTool<'a> {
    pub fn new(log: &'a dyn JiraToGanttLog) -> JiraToGanttTool {
        JiraToGanttTool { log }
    }

    pub fn run(
        self: &mut Self,
        args: impl IntoIterator<Item = std::ffi::OsString>,
    ) -> Result<(), Box<dyn Error>> {
        let cli = match Cli::try_parse_from(args) {
            Ok(m) => m,
            Err(err) => {
                output!(self.log, "{}", err.to_string());
                return Ok(());
            }
        };

        let chart_data = self.read_jira_csv_file(cli.get_input()?)?;

        self.write_chart_data_file(cli.get_output()?, &chart_data)?;

        Ok(())
    }

    fn write_chart_data_file(
        &self,
        mut writer: Box<dyn Write>,
        chart_data: &ChartData,
    ) -> Result<(), Box<dyn Error>> {
        write!(writer, "{}", json5::to_string(&chart_data)?)?;

        Ok(())
    }

    fn read_jira_csv_file(&self, reader: Box<dyn Read>) -> Result<ChartData, Box<dyn Error>> {
        let mut reader = csv::Reader::from_reader(reader);
        let mut resources: Vec<String> = vec![];
        let mut resource_items: Vec<Vec<ItemData>> = vec![];
        let headers = reader.headers().cloned().ok();

        for byte_record in reader.byte_records() {
            let byte_record: ByteRecord = byte_record?;
            let string_record: StringRecord = StringRecord::from_byte_record_lossy(byte_record);
            let record: JiraRecord = string_record.deserialize(headers.as_ref())?;

            if record.key.is_empty() {
                continue;
            }

            let mut start_date = Some(NaiveDate::parse_from_str(
                &record.created,
                "%-d/%b/%y %I:%M %p",
            )?);
            let resource_index;

            // Update resources and get the index into the array
            if let Some(index) = resources.iter().position(|s| *s == record.assignee) {
                resource_index = index;
                start_date = None;
            } else {
                resource_index = resources.len();
                resources.push(record.assignee.to_owned());
                resource_items.push(vec![]);
            }

            let mut duration: Option<i64> = None;

            if let Some(seconds) = record.original_estimate {
                duration = Some((((seconds + 1) as f32) / JIRA_DAY_IN_SECONDS).ceil() as i64);
            }

            resource_items[resource_index].push(ItemData {
                title: record.key.to_owned(),
                start_date,
                duration,
                resource_index: Some(resource_index),
                open: Some(record.status != "Closed"),
            });
        }

        // Turn empty resource into 'unassigned'
        if let Some(index) = resources.iter().position(|s| s.is_empty()) {
            resources[index] = "unassigned".to_owned();
        }

        Ok(ChartData {
            title: "".to_owned(),
            resources,
            marked_date: None,
            items: resource_items
                .iter()
                .flatten()
                .cloned()
                .collect::<Vec<ItemData>>(),
        })
    }
}
