use failure::Fallible;
use prettytable::{cell, row};
use structopt::StructOpt;

use collab::repo_collabs;
use vulns::repo_vulns;

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
    let mut table = prettytable::Table::new();

    table.add_row(row!(b => "repo", "admins"));

    for repo in repo_collabs(&cli.org, &cli.oauth_token)? {
        if !repo.is_archived {
            table.add_row(row![repo.name, fmt_admins(&repo)]);
        }
    }

    table.printstd();
    Ok(())
}

fn fmt_admins(repo: &collab::CollabRepo) -> String {
    let mut lines = vec![];
    for collab in repo.admins() {
        if collab.is_explicit_admin() {
            lines.push(collab.login.to_owned());
        }
    }
    lines.join("\n")
}

fn cmd_vulns(cli: &Cli) -> Fallible<()> {
    let mut table = prettytable::Table::new();

    table.add_row(row!(b => "repo", "archived", "vulns"));

    for repo in repo_vulns(&cli.org, &cli.oauth_token)? {
        if !repo.vulns.is_empty() {
            // println!("{}: {}", repo.name, repo.vulns.len());
            table.add_row(row![repo.name, repo.is_archived, fmt_vulns(&repo.vulns)]);
        }
    }

    table.printstd();
    Ok(())
}

fn fmt_vulns(vulns: &[vulns::VulnInfo]) -> String {
    let mut lines = vec![];
    for vuln in vulns {
        lines.push(fmt_vuln(vuln));
    }
    lines.join("\n")
}

fn fmt_vuln(vuln: &vulns::VulnInfo) -> String {
    format!("{}: {} {} ({}) {}", vuln.ecosystem, vuln.package, vuln.current_requirements, vuln.vulnerable_range, vuln.severity)
}
