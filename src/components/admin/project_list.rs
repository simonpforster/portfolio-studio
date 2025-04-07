use std::sync::Arc;
use crate::repository::project_repository::{Project, ProjectRepository};
use handlebars::{Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext};

pub(crate) struct ProjectList {
    project_repository: Arc<ProjectRepository>,
}

impl ProjectList {
    pub(crate) fn new(project_repository: Arc<ProjectRepository>) -> ProjectList {
        ProjectList { project_repository }
    }
}


impl HelperDef for ProjectList {
    fn call<'reg: 'rc, 'rc>(&self,
                            helper: &Helper,
                            hbs: &Handlebars,
                            _: &Context,
                            _rc: &mut RenderContext,
                            out: &mut dyn Output) -> HelperResult {

        let projects: Vec<Project> = self.project_repository.get_projects();
        
        out.write("<ul>")?;
        
        projects.iter().for_each(|project| {
            let _ = out.write(&format!("<li><a href='./admin-panel?project={}'>{}</a></li>", project.project_name, project.project_name));
        });
        out.write("</ul>")?;
        
        

        Ok(())
    }
}