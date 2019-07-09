use failure::Fallible;
use prettytable::{cell, row, Row};
use structopt::StructOpt;

use common::RowItem;
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

    /// Command to run
    #[structopt(subcommand)]
    cmd: Option<Command>,
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
    display_table(row!(b => "repo", "admins"), &repos, &());
    Ok(())
}

fn cmd_vulns(cli: &Cli) -> Fallible<()> {
    let mut repos = repo_vulns(&cli.org, &cli.oauth_token)?;
    vulns::VulnRepo::sort_vec(&mut repos);
    display_table(row!(b => "repo", "archived", "vulns"), &repos, &());
    Ok(())
}

fn display_table<T: RowItem>(header: Row, items: &[T], dopts: &T::DisplayOpts) {
    let mut table = prettytable::Table::new();
    table.add_row(header);
    for item in items {
        table.add_row(item.table_row(dopts));
    }

    table.printstd();
}
