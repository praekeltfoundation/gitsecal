use failure::{format_err, Fallible};
use graphql_client::GraphQLQuery;
use prettytable::{cell, row, Row};

use crate::common::RowItem;
use crate::gql_utils::Querier;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "github-schema.json",
    query_path = "src/vulns.graphql",
    response_derives = "Debug"
)]
struct RepoVulns;


use repo_vulns::{
    RepoVulnsOrganizationRepositories as RVOR,
    RepoVulnsOrganizationRepositoriesNodes as RVORN,
    RepoVulnsOrganizationRepositoriesNodesVulnerabilityAlerts as RVORNVA,
    RepoVulnsOrganizationRepositoriesNodesVulnerabilityAlertsNodes as RVORNVAN,
};


#[derive(Debug, Clone)]
pub struct VulnInfo {
    pub ecosystem: String,
    pub package: String,
    pub current_requirements: String,
    pub vulnerable_range: String,
    pub severity: String,
}

#[derive(Debug, Clone)]
pub struct VulnRepo {
    pub name: String,
    pub is_archived: bool,
    pub vulns: Vec<VulnInfo>,
}


impl RowItem for VulnRepo {
    type CmpKey = String;
    type DisplayOpts = ();

    fn cmp_key(&self) -> Self::CmpKey {
        self.name.clone()
    }

    fn table_row(&self, _opts: &Self::DisplayOpts) -> Row {
        let mut vulns = vec![];
        for vuln in &self.vulns {
            vulns.push(format!("{}: {} {} ({}) {}",
                               vuln.ecosystem,
                               vuln.package,
                               vuln.current_requirements,
                               vuln.vulnerable_range,
                               vuln.severity,
            ));
        }
        row![self.name, self.is_archived, vulns.join("\n")]
    }
}


pub fn repo_vulns(org: &str, token: &str) -> Fallible<Vec<VulnRepo>> {
    let querier = Querier::new(token)
        .header("Accept", "application/vnd.github.vixen-preview+json");
    let mut repos = vec![];

    let mut cursor = None;
    loop {
        let org_repos = rv_query(&querier, org, cursor)?;
        collect_repos(&mut repos, &org_repos)?;
        cursor = get_cursor(&org_repos);
        // println!("Cursor: {:?}", cursor);
        if cursor.is_none() { break }
    }

    Ok(repos)
}

fn get_cursor(org_repos: &RVOR) -> Option<String> {
    if org_repos.page_info.has_next_page {
       org_repos.page_info.end_cursor.clone()
    } else {
        None
    }
}

fn collect_repos(repos: &mut Vec<VulnRepo>, org_repos: &RVOR) -> Fallible<()> {
    let nodes = org_repos.nodes.as_ref();
    for node in nodes.unwrap_or(&Vec::<Option<RVORN>>::new()) {
        let repo = node.as_ref().unwrap();
        let vulns = get_repo_vulns(repo.vulnerability_alerts.as_ref().unwrap())?;
        let vr = VulnRepo {
            name: repo.name.clone(),
            is_archived: repo.is_archived,
            vulns,
        };
        repos.push(vr);
    }
    Ok(())
}

fn get_repo_vulns(vuln_alerts: &RVORNVA) -> Fallible<Vec<VulnInfo>> {
    let mut vis = vec![];
    let nodes = vuln_alerts.nodes.as_ref();
    for node in nodes.unwrap_or(&Vec::<Option<RVORNVAN>>::new()) {
        let alert = node.as_ref().unwrap();
        let current_requirements = alert.vulnerable_requirements.as_ref().unwrap().clone();
        let vuln = alert.security_vulnerability.as_ref().unwrap();
        let ecosystem = enum_to_string(&vuln.package.ecosystem)?;
        let package = vuln.package.name.clone();
        let vulnerable_range = vuln.vulnerable_version_range.clone();
        let severity = enum_to_string(&vuln.severity)?;
        let vi = VulnInfo {
            ecosystem,
            package,
            current_requirements,
            vulnerable_range,
            severity,
        };
        vis.push(vi);
    }
    Ok(vis)
}

fn enum_to_string<T: serde::Serialize>(x: &T) -> Fallible<String> {
    Ok(serde_json::from_str(&serde_json::to_string(x)?)?)
}

fn rv_query(querier: &Querier, org: &str, cursor: Option<String>) -> Fallible<RVOR> {
    let q = RepoVulns::build_query(repo_vulns::Variables {
        org: org.to_string(),
        cursor,
    });

    let rd: repo_vulns::ResponseData = querier.query(&q)?;
    let org_repos = rd
        .organization.ok_or_else(|| format_err!("missing org in response data"))?
        .repositories;
    Ok(org_repos)
}
