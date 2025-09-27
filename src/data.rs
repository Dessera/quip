use crate::{QuipError, QuipResult};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub users: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackendData {
    users: Vec<User>,
    groups: Vec<Group>,
}

impl BackendData {
    pub fn new(users: Vec<User>, groups: Vec<Group>) -> Self {
        Self { users, groups }
    }

    pub async fn from_file(path: impl AsRef<str>) -> QuipResult<Self> {
        let path = path.as_ref();

        let mut fd = File::open(path).await?;
        let mut contents = vec![];

        fd.read_to_end(&mut contents).await?;

        Ok(serde_json::from_slice::<Self>(contents.as_slice())?)
    }
}

#[derive(Debug)]
pub struct QueryGroup(pub Group, pub HashSet<Arc<User>>);

/// Fast query map for [`BackendData`].
#[derive(Debug)]
pub struct BackendQueryData {
    pub users: HashMap<String, Arc<User>>,
    pub groups: HashMap<String, QueryGroup>,
}

impl TryFrom<BackendData> for BackendQueryData {
    type Error = QuipError;

    fn try_from(value: BackendData) -> QuipResult<Self> {
        let users: HashMap<String, Arc<User>> = value
            .users
            .into_iter()
            .map(|user| (user.name.clone(), Arc::new(user)))
            .collect();

        let groups: QuipResult<HashMap<String, QueryGroup>> = value
            .groups
            .into_iter()
            .map(|group| {
                let grp_name = group.name.clone();
                let grp_query = group_to_query(group, &grp_name, &users)?;
                Ok((grp_name, grp_query))
            })
            .collect();

        Ok(Self {
            users,
            groups: groups?,
        })
    }
}

fn group_to_query(
    group: Group,
    grp_name: &str,
    users: &HashMap<String, Arc<User>>,
) -> QuipResult<QueryGroup> {
    let grp_users: QuipResult<HashSet<Arc<User>>> = group
        .users
        .iter()
        .map(|grp_user| match users.get(grp_user) {
            Some(user) => Ok(user.clone()),
            None => {
                return Err(QuipError::NotFound(format!(
                    "User named {} required in {} does not exist",
                    grp_user, grp_name
                )));
            }
        })
        .collect();

    Ok(QueryGroup(group, grp_users?))
}
