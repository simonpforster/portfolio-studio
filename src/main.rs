pub mod components;
pub mod observability;
pub mod repository;

use axum::{extract::State, http::StatusCode, middleware, response::{Html, IntoResponse}, routing::get, Router};
use handlebars::Handlebars;
use serde_json::json;
use std::{env, net::SocketAddr, sync::Arc};
use axum::extract::Query;
use axum::response::Redirect;
use firestore::FirestoreDb;
use serde::Deserialize;
use tower_http::services::ServeDir;
use tracing::{info, instrument, warn};
use crate::components::admin::project_list::ProjectList;
use crate::components::projects::project_description::ProjectDescription;
use crate::components::projects::project_tags::ProjectTags;
use crate::observability::init_tracing;
use crate::observability::propagators::extract_context;
use crate::repository::project_repository::ProjectRepository;

// App state that will be shared across all routes
#[derive(Debug)]
struct AppState {
    hbs: Handlebars<'static>,
    project_repository: Arc<ProjectRepository>
}


#[tokio::main]
async fn main() {
    let service_name: String = env::var("K_SERVICE").unwrap_or("oliviazuo-portfolio".into());
    let gcp_project_id: String = env::var("GCP_PROJECT_ID").expect("env var GCP_PROJECT_ID not configured");

    // Initialize tracing for nice logging
    let _ = init_tracing(service_name, gcp_project_id.clone()).await;

    let port: u16 = env::var("PORT").unwrap_or_else(|_| {
        warn!("env var PORT not configured, defaulting to 8080");
        "8080".into()
    }).parse::<u16>().expect("env var PORT must be a valid number");

    // Init DB
    let db: FirestoreDb = FirestoreDb::new(&gcp_project_id).await.expect("Could not initiate DB client");

    let proj_repo = ProjectRepository::new(db);
    proj_repo.fill_cache().await;

    let thingy = Arc::new(proj_repo);

    // Create and register templates
    let mut hbs = Handlebars::new();

    // Define description helper
    hbs.register_helper("project_description", Box::new(ProjectDescription::new(thingy.clone())));

    hbs.register_helper("project_tags", Box::new(ProjectTags::new(thingy.clone())));

    hbs.register_helper("project_list", Box::new(ProjectList::new(thingy.clone())));


    // Define templates
    hbs.register_template_file("project-list", "templates/admin-panel/project-list.hbs")
        .expect("Failed to register project-list template");
    hbs.register_template_file("project-edit", "templates/admin-panel/project-edit.hbs")
        .expect("Failed to register project-edit template");


    // Create shared application state
    let state = Arc::new(AppState { hbs, project_repository: thingy.clone() });

    // Set up our application with routes
    let app = Router::new()
        .route("/admin-panel", get(admin_handler)) // to be turned into a proxy at a later date
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
        .layer(middleware::from_fn(extract_context));

    // Run our application
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("listening on {}", addr);
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize, Debug)]
struct AdminQuery {
    project: Option<String>,
}

#[instrument]
async fn admin_handler(State(state): State<Arc<AppState>>, project: Query<AdminQuery>) -> impl IntoResponse {
    let project = project.0.project;

    match project {
        Some(project) => {

            let d = state.project_repository.get_projects().iter().map(|p| {p.project_name});

            let data = json!({
                "project_name": project
            });

            match state.hbs.render("project-edit", &data) {
                Ok(html) => Html(html).into_response(),
                Err(err) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Template error: {}", err),
                )
                    .into_response(),
            }
        }
        None => {
            let data = json!({
            });

            match state.hbs.render("project-list", &data) {
                Ok(html) => Html(html).into_response(),
                Err(err) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Template error: {}", err),
                )
                    .into_response(),
            }
        }
    }
}
