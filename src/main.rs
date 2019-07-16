use failure::{format_err, Fallible};
use prettytable::{cell, row, Row};
use structopt::StructOpt;

use common::{CommonOpts, DisplayOpts, RowItem};
use collab::repo_collabs;
use vulns::repo_vulns;

mod common;
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

    /// Output format, one of {table, oneline, csv}
    #[structopt(long, env = "OUTPUT_FORMAT", default_value = "table")]
    output_format: OutputFormat,

    /// Command to run
    #[structopt(subcommand)]
    cmd: Option<Command>,
}

#[derive(Clone, Copy, Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum OutputFormat {
    Table,
    Oneline,
    Csv,
}

impl OutputFormat {
    fn common_opts(&self) -> CommonOpts {
        match self {
            OutputFormat::Table => CommonOpts {
                multiline: true,
                borders: true,
                csv: false,
            },
            OutputFormat::Oneline => CommonOpts {
                multiline: false,
                borders: false,
                csv: false,
            },
            OutputFormat::Csv => CommonOpts {
                multiline: false,
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
            "oneline" => Ok(OutputFormat::Oneline),
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
    let mut repos = repo_collabs(&cli.org, &cli.oauth_token)?;
    collab::CollabRepo::sort_vec(&mut repos);
    let dopts = collab::CRDisplayOpts::new(cli.output_format.common_opts());
    display_table(row!(b => "repo", "admins"), &repos, &dopts)
}

fn cmd_vulns(cli: &Cli) -> Fallible<()> {
    let mut repos = repo_vulns(&cli.org, &cli.oauth_token)?;
    vulns::VulnRepo::sort_vec(&mut repos);
    let dopts = vulns::VRDisplayOpts::new(cli.output_format.common_opts());
    display_table(row!(b => "repo", "archived", "vulns"), &repos, &dopts)
}


fn display_table<T: RowItem>(header: Row, items: &[T], dopts: &T::DisplayOpts) -> Fallible<()> {
    let mut table = prettytable::Table::new();
    table.add_row(header);
    for item in items {
        table.add_row(item.table_row(dopts));
    }

    if !dopts.common_opts().borders {
        table.set_format(*prettytable::format::consts::FORMAT_CLEAN);
    }

    if dopts.common_opts().csv {
        table.to_csv(std::io::stdout())?;
    } else {
        table.printstd();
    }
    Ok(())
}
