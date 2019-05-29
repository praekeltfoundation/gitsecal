use prettytable::{cell, row};
use structopt::StructOpt;

use vulns::repo_vulns;

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
}

fn main() -> Result<(), failure::Error> {
    let cli = Cli::from_args();

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
