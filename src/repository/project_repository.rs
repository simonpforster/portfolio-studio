use std::sync::RwLock;
use firestore::{path, FirestoreDb, FirestoreQueryDirection, FirestoreResult};
use futures_core::stream::BoxStream;
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ProjectRepository {
    db: FirestoreDb,
    cache: RwLock<Vec<Project>>
}

impl ProjectRepository {
    pub fn new(db: FirestoreDb) -> Self {
        ProjectRepository {
            db,
            cache: RwLock::new(Vec::new())
        }
    }

    pub fn get_project(&self, project_name: &str) -> Option<Project> {
        self.cache.read().unwrap().iter().find_map(|p| {
            if p.project_name == project_name {
                Some(p.clone())
            } else { None }
        })
    }

    pub fn get_projects(&self) -> Vec<Project> {
        self.cache.read().unwrap().to_vec()
    }

    pub async fn fill_cache(&self) {
        let projects = self.get_projects_from_firestore().await;
        match projects {
            Ok(projects) => {
                let mut cache = self.cache.write().unwrap();
                *cache = projects;
            }
            Err(_) => {}
        }
    }



    pub async fn get_projects_from_firestore(&self) -> FirestoreResult<Vec<Project>> {
        let stream: BoxStream<FirestoreResult<Project>> = self.db.fluent().select()
            .from("projects")
            .filter(|q| {
                q.field("owner").eq("Olivia Zuo")
            })
            .order_by([(
                path!(Project::year),
                FirestoreQueryDirection::Descending,
            )])
            .obj()
            .stream_query_with_errors()
            .await?;
        let projects: Vec<Project> = stream.try_collect().await?;
        Ok(projects)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    #[serde(rename = "projectName")]
    pub(crate) project_name: String,
    owner: String,
    year: u16,
    #[serde(rename = "type")]
    project_type: String,
    pub description: String,
    pub tags: Vec<String>,
}