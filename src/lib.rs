/// Generate a Gantt chart
use chrono::NaiveDate;
use clap::Parser;
use core::fmt::Arguments;
use csv;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

mod log_macros;

const JIRA_DAY_IN_SECONDS: f32 = 8.0 * 60.0 * 60.0;
static RESOURCE_COLORS: [u32; 14] = [
    0xff1abc9c, 0xff3498db, 0xff9b59b6, 0xffe67e22, 0xff2ecc71, 0xffe74c3c, 0xfff39c12, 0xff27ae60,
    0xff2980b9, 0xfff1c40f, 0xffd35400, 0xff8e44ad, 0xffc0392b, 0xff16a085,
];

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Cli {
    /// Specify the JSON data file
    #[clap(value_name = "INPUT_FILE")]
    input_file: PathBuf,

    #[clap(long = "output", short, value_name = "OUTPUT_FILE")]
    output_file: PathBuf,
}

pub trait JiraToGanttLog {
    fn output(self: &Self, args: Arguments);
    fn warning(self: &Self, args: Arguments);
    fn error(self: &Self, args: Arguments);
}

pub struct JiraToGanttTool<'a> {
    log: &'a dyn JiraToGanttLog,
}

// TODO: This should come from the gantt_chart tool as a library
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ItemData {
    title: String,
    duration: Option<i64>,
    #[serde(rename = "startDate", skip_serializing_if = "Option::is_none")]
    start_date: Option<NaiveDate>,
    #[serde(rename = "resource")]
    resource_index: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ResourceData {
    #[allow(dead_code)]
    title: String,
    #[serde(rename = "color")]
    color_hex: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ChartData {
    #[allow(dead_code)]
    title: String,
    resources: Vec<ResourceData>,
    items: Vec<ItemData>,
}

#[derive(Deserialize, Debug)]
struct JiraRecord {
    #[serde(rename = "Key")]
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

        let chart_data = self.read_jira_csv_file(cli.input_file)?;

        self.write_chart_data_file(cli.output_file, &chart_data)?;

        Ok(())
    }

    fn write_chart_data_file(
        self: &Self,
        json_path: PathBuf,
        chart_data: &ChartData,
    ) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(json_path)?;

        write!(file, "{}", json5::to_string(&chart_data)?)?;

        Ok(())
    }

    fn read_jira_csv_file(self: &Self, csv_path: PathBuf) -> Result<ChartData, Box<dyn Error>> {
        let file = File::open(csv_path)?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut resources: Vec<ResourceData> = vec![];
        let mut resource_items: Vec<Vec<ItemData>> = vec![];

        for result in rdr.deserialize() {
            let record: JiraRecord = result?;

            if record.key.is_empty() {
                continue;
            }

            let mut start_date = Some(NaiveDate::parse_from_str(
                &record.created,
                "%-m/%-d/%y %H:%M",
            )?);
            let resource_index;

            if let Some(index) = resources.iter().position(|rd| rd.title == record.assignee) {
                resource_index = index;
                start_date = None;
            } else {
                resource_index = resources.len();
                resources.push(ResourceData {
                    title: record.assignee.to_owned(),
                    color_hex: RESOURCE_COLORS[resource_index],
                });
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
            });
        }

        Ok(ChartData {
            title: "".to_owned(),
            resources,
            items: resource_items
                .iter()
                .flatten()
                .cloned()
                .collect::<Vec<ItemData>>(),
        })
    }
}
