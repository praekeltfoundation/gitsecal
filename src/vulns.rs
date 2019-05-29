use failure::{format_err, Fallible};

use graphql_client::{GraphQLQuery, Response};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "github-schema.json",
    query_path = "src/vulns.graphql",
    response_derives = "Debug"
)]
struct RepoVulns;


type RVOR = repo_vulns::RepoVulnsOrganizationRepositories;
type RVORN = repo_vulns::RepoVulnsOrganizationRepositoriesNodes;
type RVORNVA = repo_vulns::RepoVulnsOrganizationRepositoriesNodesVulnerabilityAlerts;
type RVORNVAN = repo_vulns::RepoVulnsOrganizationRepositoriesNodesVulnerabilityAlertsNodes;

#[derive(Debug, Clone)]
pub struct VulnInfo {
    pub ecosystem: String,
    pub package: String,
    pub current_requirements: String,
    pub vulnerable_range: String,
}

#[derive(Debug, Clone)]
pub struct VulnRepo {
    pub name: String,
    pub is_archived: bool,
    pub vulns: Vec<VulnInfo>,
}

pub fn repo_vulns(org: &str, token: &str) -> Fallible<Vec<VulnRepo>> {
    let mut repos = vec![];

    let mut cursor = None;
    loop {
        let org_repos = rv_query(org, token, cursor)?;
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
        let name = repo.name.clone();
        let is_archived = repo.is_archived.clone();
        let vulns = get_repo_vulns(repo.vulnerability_alerts.as_ref().unwrap())?;
        let vr = VulnRepo {
            name,
            is_archived,
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
        let ecosystem = eco_to_string(&vuln.package.ecosystem)?;
        let package = vuln.package.name.clone();
        let vulnerable_range = vuln.vulnerable_version_range.clone();
        let vi = VulnInfo {
            ecosystem,
            package,
            current_requirements,
            vulnerable_range,
        };
        vis.push(vi);
    }
    Ok(vis)
}

fn eco_to_string(eco: &repo_vulns::SecurityAdvisoryEcosystem) -> Fallible<String> {
    Ok(serde_json::from_str(&serde_json::to_string(eco)?)?)
}

fn rv_query(org: &str, token: &str, cursor: Option<String>) -> Fallible<RVOR> {
    let q = RepoVulns::build_query(repo_vulns::Variables {
        org: org.to_string(),
        cursor: cursor,
    });

    let client = reqwest::Client::new();

    let mut res = client
        .post("https://api.github.com/graphql")
        .header("Accept", "application/vnd.github.vixen-preview+json")
        .bearer_auth(token)
        .json(&q)
        .send()?;

    let response_body: Response<repo_vulns::ResponseData> = res.json()?;

    // println!("Response: {:?}", response_body);

    if let Some(errors) = response_body.errors {
        println!("there are errors:");

        for error in &errors {
            println!("{:?}", error);
        }
    }

    let org_repos = response_body
        .data.ok_or(format_err!("missing response data"))?
        .organization.ok_or(format_err!("missing org in response data"))?
        .repositories;
    Ok(org_repos)
}
