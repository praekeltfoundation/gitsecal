use failure::{format_err, Fallible};
use structopt::StructOpt;

use display::{CommonOpts, printstd};
use collab::repo_collabs;
use vulns::repo_vulns;

mod common;
mod display;
mod gql_utils;
mod collab;
mod vulns;

/// A little tool for finding github security alerts for all repos in an org
#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct Cli {
    /// Github OAuth token
    #[structopt(long, env = "GH_OAUTH_TOKEN")]
    oauth_token: String,

    /// Github organization to scan
    #[structopt(long, env = "GH_ORG")]
    org: String,

    /// Output format, one of {table, table-clean, csv}
    #[structopt(long, env = "OUTPUT_FORMAT", default_value = "table")]
    output_format: OutputFormat,

    /// Compress each output row to a single line
    #[structopt(long)]
    output_oneline: bool,

    /// Command to run
    #[structopt(subcommand)]
    cmd: Option<Command>,
}

#[derive(Clone, Copy, Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum OutputFormat {
    Table,
    TableClean,
    Csv,
}

impl Cli {
    fn common_opts(&self) -> CommonOpts {
        match self.output_format {
            OutputFormat::Table => CommonOpts {
                multiline: !self.output_oneline,
                borders: true,
                csv: false,
            },
            OutputFormat::TableClean => CommonOpts {
                multiline: !self.output_oneline,
                borders: false,
                csv: false,
            },
            OutputFormat::Csv => CommonOpts {
                multiline: !self.output_oneline,
                borders: false,
                csv: true,
            },
        }
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "table" => Ok(OutputFormat::Table),
            "table-clean" => Ok(OutputFormat::TableClean),
            "csv" => Ok(OutputFormat::Csv),
            _ => Err(format_err!("Unknown format: {}", s)),
        }
    }
}

#[derive(Clone, Copy, Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum Command {
    Vulns,
    Admins,
}

fn main() -> Fallible<()> {
    let cli = Cli::from_args();

    match cli.cmd.unwrap_or(Command::Vulns) {
        Command::Vulns => cmd_vulns(&cli),
        Command::Admins => cmd_admins(&cli),
    }
}

fn cmd_admins(cli: &Cli) -> Fallible<()> {
    let mut content = repo_collabs(&cli.org, &cli.oauth_token)?;
    content.sort_on("repo");
    printstd(content, cli.common_opts())
}

fn cmd_vulns(cli: &Cli) -> Fallible<()> {
    let mut content = repo_vulns(&cli.org, &cli.oauth_token)?;
    content.sort_on("repo");
    printstd(content, cli.common_opts())
}
