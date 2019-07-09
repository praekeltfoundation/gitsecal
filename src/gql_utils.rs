use std::fmt;

use failure::{format_err, Fallible};
use graphql_client::{QueryBody, Response};
use serde::ser::Serialize;
use serde::de::DeserializeOwned;


/// This only exists so that we can derive Debug on Querier.
#[derive(Clone, Copy)]
struct FilterFn<'a>(&'a Fn(&graphql_client::Error) -> bool);

impl fmt::Debug for FilterFn<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#FilterFn")
    }
}


/// An abstraction over a reqwest client for making graphql calls.
///
/// The internal client is shared across any clones.
#[derive(Debug, Clone)]
pub struct Querier<'a> {
    token: &'a str,
    uri: &'a str,
    headers: Vec<(&'a str, &'a str)>,
    error_filter: Option<FilterFn<'a>>,
    client: Box<reqwest::Client>,
}

impl<'a> Querier<'a> {
    pub fn new(token: &'a str) -> Self {
        Self {
            token,
            uri: "https://api.github.com/graphql",
            headers: vec![],
            error_filter: None,
            client: Box::new(reqwest::Client::new()),
        }
    }

    pub fn header(self, name: &'a str, value: &'a str) -> Self {
        let mut headers = self.headers.clone();
        headers.push((name, value));
        Self {headers, ..self}
    }

    pub fn error_filter(self, f: &'a Fn(&graphql_client::Error) -> bool) -> Self {
        Self {error_filter: Some(FilterFn(f)), ..self}
    }

    fn filter_errs(&self, errs: &'a[graphql_client::Error]) -> Vec<&'a graphql_client::Error> {
        match self.error_filter {
            None => errs.iter().collect(),
            Some(ffn) => errs.iter().filter(|e| ffn.0(*e)).collect(),
        }
    }

    pub fn query<RD: DeserializeOwned>(&self, query: &QueryBody<impl Serialize>) -> Fallible<RD> {
        let mut reqb = self.client
            .post(self.uri)
            .bearer_auth(self.token);
        for (name, value) in &self.headers {
            reqb = reqb.header(*name, *value);
        }
        let mut res = reqb.json(query).send()?;

        let body: Response<RD> = match res.json() {
            Ok(json) => json,
            Err(e) => {
                println!("Bad response: {:?}\n{:?}", e, res.text());
                return Err(e)?;
            },
        };

        // println!("Response: {:?}", body);

        if let Some(errors) = body.errors {
            let filtered = self.filter_errs(&errors);

            if !filtered.is_empty() {
                println!("there are errors:");
                for error in &filtered {
                    println!("{:?}", error);
                }
            }
        }
        body.data.ok_or_else(|| format_err!("missing response data"))
    }
}
