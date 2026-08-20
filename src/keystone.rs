use crate::client::Session;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub domain_id: String,
}

pub fn list_projects(session: &Session) -> Result<Vec<Project>, String> {
    let body = session.get("identity", "/v3/projects")?;
    let parsed: RawProjectList = serde_json::from_value(body)
        .map_err(|e| format!("unexpected response shape from Keystone: {e}"))?;

    Ok(parsed
        .projects
        .into_iter()
        .map(RawProject::into_project)
        .collect())
}

#[derive(Debug, Deserialize)]
struct RawProjectList {
    projects: Vec<RawProject>,
}

#[derive(Debug, Deserialize)]
struct RawProject {
    id: String,
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default)]
    domain_id: String,
}

fn default_true() -> bool {
    true
}

impl RawProject {
    fn into_project(self) -> Project {
        Project {
            id: self.id,
            name: self.name,
            description: if self.description.is_empty() {
                "-".to_string()
            } else {
                self.description
            },
            enabled: self.enabled,
            domain_id: self.domain_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_project_defaults_empty_description_to_dash() {
        let raw = RawProject {
            id: "1".into(),
            name: "demo".into(),
            description: String::new(),
            enabled: true,
            domain_id: "default".into(),
        };
        assert_eq!(raw.into_project().description, "-");
    }

    #[test]
    fn raw_project_keeps_real_description() {
        let raw = RawProject {
            id: "1".into(),
            name: "admin".into(),
            description: "Bootstrap project".into(),
            enabled: true,
            domain_id: "default".into(),
        };
        assert_eq!(raw.into_project().description, "Bootstrap project");
    }

    #[test]
    fn raw_project_maps_disabled() {
        let raw = RawProject {
            id: "1".into(),
            name: "old".into(),
            description: String::new(),
            enabled: false,
            domain_id: "default".into(),
        };
        assert!(!raw.into_project().enabled);
    }
}
