use failure::{format_err, Fallible};

use graphql_client::GraphQLQuery;

use crate::gql_utils::Querier;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "github-schema.json",
    query_path = "src/collab.graphql",
    response_derives = "Debug"
)]
struct RepoCollabs;


use repo_collabs::{
    RepoCollabsOrganizationRepositories as RCOR,
    RepoCollabsOrganizationRepositoriesNodes as RCORN,
    RepoCollabsOrganizationRepositoriesNodesCollaborators as RCORNC,
    RepoCollabsOrganizationRepositoriesNodesCollaboratorsEdges as RCORNCE,
    RepoCollabsOrganizationRepositoriesNodesCollaboratorsEdgesPermissionSources as RCORNCEPS,
    RepoCollabsOrganizationRepositoriesNodesCollaboratorsEdgesPermissionSourcesSource as RCORNCEPSS,
};


#[derive(Debug, Clone)]
pub struct PermSource {
    pub permission: String,
    pub source: String,
    pub source_name: String,
}

#[derive(Debug, Clone)]
pub struct CollabInfo {
    pub login: String,
    pub permission: String,
    pub sources: Vec<PermSource>,
}

#[derive(Debug, Clone)]
pub struct CollabRepo {
    pub name: String,
    pub is_archived: bool,
    pub collabs: Vec<CollabInfo>,
}


impl CollabInfo {
    pub fn is_admin(&self) -> bool {
        self.permission == "ADMIN"
    }

    pub fn is_explicit_admin(&self) -> bool {
        // While each permission source generally denotes a specific place that
        // permission is granted (a read permission from being a member of the
        // org that owns the repo, a write permission from being a member of a
        // team that has been granted write permissions, an admin permission
        // granted explicitly to the user for that repo, etc.) it seems that
        // admin permissions for org owners also come with an additional repo
        // source that admin permission. Thus, an org owner with no explicit
        // admin permission gets an org source and a repo source. An org owner
        // *with* an explicit admin permission gets one org source and two repo
        // sources.
        let mut org_admin = false;
        let mut admin_sources = 0;
        for source in &self.sources {
            if source.permission == "ADMIN" {
                admin_sources += 1;
                if source.source == "org" {
                    org_admin = true;
                }
            }
        }
        (admin_sources > 2) || (!org_admin && admin_sources > 0)
    }
}

impl CollabRepo {
    pub fn admins(&self) -> Vec<&CollabInfo> {
        self.collabs.iter().filter(|c| c.is_admin()).collect()
    }
}

pub fn repo_collabs(org: &str, token: &str) -> Fallible<Vec<CollabRepo>> {
    let querier = Querier::new(token)
        .header("Accept", "application/vnd.github.vixen-preview+json")
        .error_filter(&|e: &graphql_client::Error| {
            e.message != "Must have push access to view repository collaborators."
        });
    let mut repos = vec![];

    let mut cursor = None;
    loop {
        let org_repos = rc_query(&querier, org, cursor)?;
        collect_repos(&mut repos, &org_repos)?;
        cursor = get_cursor(&org_repos);
        println!("Cursor: {:?}", cursor);
        if cursor.is_none() { break }
    }

    Ok(repos)
}

fn get_cursor(org_repos: &RCOR) -> Option<String> {
    if org_repos.page_info.has_next_page {
       org_repos.page_info.end_cursor.clone()
    } else {
        None
    }
}

fn collect_repos(repos: &mut Vec<CollabRepo>, org_repos: &RCOR) -> Fallible<()> {
    let nodes = org_repos.nodes.as_ref();
    for node in nodes.unwrap_or(&Vec::<Option<RCORN>>::new()) {
        let repo = node.as_ref().unwrap();
        // println!("{:?}:", &repo.name);
        let collabs = if let Some(cs) = &repo.collaborators {
            get_repo_collabs(cs)?
        } else {
            vec![]
        };
        let vr = CollabRepo {
            name: repo.name.clone(),
            is_archived: repo.is_archived,
            collabs,
        };
        repos.push(vr);
    }
    Ok(())
}

fn get_repo_collabs(collabs: &RCORNC) -> Fallible<Vec<CollabInfo>> {
    let mut cis = vec![];
    let edges = collabs.edges.as_ref();
    for edge in edges.unwrap_or(&Vec::<Option<RCORNCE>>::new()) {
        // println!("  {:#?}", edge);
        let edge = edge.as_ref().unwrap();
        let login = edge.node.login.clone();
        let permission = enum_to_string(&edge.permission)?;
        let sources = if let Some(ps) = &edge.permission_sources {
            get_perm_sources(ps)?
        } else {
            vec![]
        };
        let ci = CollabInfo {
            login,
            permission,
            sources,
        };
        cis.push(ci);
    }
    Ok(cis)
}

fn get_perm_sources(perm_sources: &[RCORNCEPS]) -> Fallible<Vec<PermSource>> {
    let mut pss = vec![];
    for perm_source in perm_sources {
        // println!("  {:#?}", perm_source);
        let permission = enum_to_string(&perm_source.permission)?;
        let (source, source_name) = match perm_source.source {
            RCORNCEPSS::Organization(ref org) => ("org".to_owned(), org.login.clone()),
            RCORNCEPSS::Repository(ref repo) => ("repo".to_owned(), repo.name.clone()),
            RCORNCEPSS::Team(ref team) => ("team".to_owned(), team.name.clone()),
        };
        let ps = PermSource {
            permission,
            source,
            source_name,
        };
        pss.push(ps);
    }
    Ok(pss)
}

fn enum_to_string<T: serde::Serialize>(x: &T) -> Fallible<String> {
    Ok(serde_json::from_str(&serde_json::to_string(x)?)?)
}

fn rc_query(querier: &Querier, org: &str, cursor: Option<String>) -> Fallible<RCOR> {
    let q = RepoCollabs::build_query(repo_collabs::Variables {
        org: org.to_string(),
        cursor,
    });

    let rd: repo_collabs::ResponseData = querier.query(&q)?;
    let org_repos = rd
        .organization.ok_or_else(|| format_err!("missing org in response data"))?
        .repositories;
    Ok(org_repos)
}
