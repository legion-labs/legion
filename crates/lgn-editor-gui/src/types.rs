use std::rc::Rc;

use log::debug;
use reqwest::{Method, Url};
use serde::Deserialize;

use crate::errors::Error;
use crate::utils::tree::{Tree, TreeVisitor};

static BASE_URL: &str = "https://jsonplaceholder.typicode.com";

pub trait Request {
    type Options;
    type Output;
    type Error;

    fn request(options: Self::Options) -> reqwest::Request;
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Todo {
    pub user_id: u32,
    pub id: u32,
    pub title: String,
    pub completed: bool,
}

pub struct TodosRequest;

impl Request for TodosRequest {
    type Options = Rc<String>;
    type Output = Vec<Todo>;
    type Error = Error;

    fn request(token: Self::Options) -> reqwest::Request {
        let base_url: Url = BASE_URL.parse().unwrap();

        let url = base_url.join("todos").unwrap();

        let client = reqwest::Client::new();

        let mut req = client.request(Method::GET, url);

        req = req.bearer_auth(&*token);

        req.build().unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ResourceDescription {
    id: String,
    pub path: String,
    version: u32,
    #[serde(rename = "type")]
    type_: String,
}

struct ExampleResourceEntryTreeVisitor;

impl TreeVisitor<String, ResourceEntry> for ExampleResourceEntryTreeVisitor {
    fn visit_value(&self, value: &ResourceEntry, _depth: u8) {
        let path = match value {
            ResourceEntry::Root => "root",
            ResourceEntry::Folder { name, .. } => name,
            ResourceEntry::Entry { value, .. } => &value.path,
        };

        debug!("{}", path);
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct NextSearchToken {
    pub next_search_token: String,
    pub total: u64,
    pub resource_descriptions: Vec<ResourceDescription>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResourceEntry {
    Root,
    Folder {
        name: String,
    },
    Entry {
        name: String,
        value: ResourceDescription,
    },
}

impl ResourceEntry {
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        match self {
            Self::Root => "root",
            Self::Folder { name, .. } => name,
            Self::Entry { name, .. } => name,
        }
    }
}

pub type TreeResourceEntry = Tree<String, Rc<ResourceEntry>>;

impl From<Rc<NextSearchToken>> for TreeResourceEntry {
    fn from(next_search_token: Rc<NextSearchToken>) -> Self {
        let mut tree = Tree::from_value(Rc::new(ResourceEntry::Root));

        for resource in &next_search_token.resource_descriptions {
            let path = resource.path.as_str()[1..].to_string();

            let keys = path.split('/').map(Into::into).collect::<Vec<String>>();

            let name = keys.last().unwrap().to_string();

            tree.insert_at_or_else(
                keys,
                Rc::new(ResourceEntry::Entry {
                    name,
                    value: resource.clone(),
                }),
                |key| Rc::new(ResourceEntry::Folder { name: key.clone() }),
            );
        }

        tree
    }
}

pub struct NextSearchTokenRequest;

impl Request for NextSearchTokenRequest {
    type Options = Rc<String>;
    type Output = NextSearchToken;
    type Error = Error;

    fn request(token: Self::Options) -> reqwest::Request {
        let client = reqwest::Client::new();

        let mut req = client.request(
            Method::GET,
            "http://[::1]:5051/v1/spaces/0/workspaces/0/resources/search/?token=",
        );

        req = req.bearer_auth(&*token);

        req.build().unwrap()
    }
}
