use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

/// Everything needed to authenticate against Keystone: either loaded from
/// `clouds.yaml` / `OS_*` environment variables (the standard OpenStack
/// client convention, so stackboard picks up whatever's already configured
/// for the `openstack` CLI), or gathered by the interactive setup wizard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudAuth {
    pub auth_url: String,
    pub username: String,
    pub password: String,
    pub project_name: String,
    #[serde(default = "default_domain")]
    pub user_domain_name: String,
    #[serde(default = "default_domain")]
    pub project_domain_name: String,
    pub region_name: Option<String>,
}

fn default_domain() -> String {
    "Default".to_string()
}

#[derive(Debug, Deserialize)]
struct CloudsFile {
    clouds: HashMap<String, CloudEntry>,
}

#[derive(Debug, Deserialize)]
struct CloudEntry {
    auth: AuthSection,
    region_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AuthSection {
    auth_url: String,
    username: Option<String>,
    password: Option<String>,
    project_name: Option<String>,
    #[serde(default = "default_domain")]
    user_domain_name: String,
    #[serde(default = "default_domain")]
    project_domain_name: String,
}

/// The stackboard-managed config file, written by the setup wizard. A real
/// clouds.yaml-shaped file so the `openstack` CLI can read it too if
/// pointed at it, under a single `stackboard` entry.
fn stackboard_config_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".config/stackboard/clouds.yaml"))
}

fn clouds_yaml_search_paths() -> Vec<PathBuf> {
    let mut paths = vec![PathBuf::from("clouds.yaml")];
    if let Some(p) = stackboard_config_path() {
        paths.push(p);
    }
    if let Ok(home) = std::env::var("HOME") {
        paths.push(PathBuf::from(home).join(".config/openstack/clouds.yaml"));
    }
    paths.push(PathBuf::from("/etc/openstack/clouds.yaml"));
    paths
}

fn load_clouds_yaml(cloud_name: Option<&str>) -> Option<CloudAuth> {
    for path in clouds_yaml_search_paths() {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(file) = serde_yaml::from_str::<CloudsFile>(&text) else {
            continue;
        };
        let entry = match cloud_name {
            Some(name) => file.clouds.get(name),
            None if file.clouds.len() == 1 => file.clouds.values().next(),
            None => None, // multiple clouds and no OS_CLOUD to disambiguate
        };
        if let Some(entry) = entry
            && let (Some(username), Some(password), Some(project_name)) = (
                entry.auth.username.clone(),
                entry.auth.password.clone(),
                entry.auth.project_name.clone(),
            )
        {
            return Some(CloudAuth {
                auth_url: entry.auth.auth_url.clone(),
                username,
                password,
                project_name,
                user_domain_name: entry.auth.user_domain_name.clone(),
                project_domain_name: entry.auth.project_domain_name.clone(),
                region_name: entry.region_name.clone(),
            });
        }
    }
    None
}

fn load_from_env() -> Option<CloudAuth> {
    Some(CloudAuth {
        auth_url: std::env::var("OS_AUTH_URL").ok()?,
        username: std::env::var("OS_USERNAME").ok()?,
        password: std::env::var("OS_PASSWORD").ok()?,
        project_name: std::env::var("OS_PROJECT_NAME").ok()?,
        user_domain_name: std::env::var("OS_USER_DOMAIN_NAME").unwrap_or_else(|_| default_domain()),
        project_domain_name: std::env::var("OS_PROJECT_DOMAIN_NAME")
            .unwrap_or_else(|_| default_domain()),
        region_name: std::env::var("OS_REGION_NAME").ok(),
    })
}

/// Resolve credentials the same way the `openstack` CLI does: `OS_*`
/// environment variables first (so an already-`source`d openrc.sh just
/// works), then `clouds.yaml` (`OS_CLOUD` picks the entry when there's more
/// than one), then `None` if nothing is configured — the caller should run
/// the setup wizard in that case.
pub fn discover() -> Option<CloudAuth> {
    if let Some(auth) = load_from_env() {
        return Some(auth);
    }
    let cloud_name = std::env::var("OS_CLOUD").ok();
    load_clouds_yaml(cloud_name.as_deref())
}

/// Prompts on stdin/stdout for everything needed to authenticate, then
/// writes it to `~/.config/stackboard/clouds.yaml` under a `stackboard`
/// entry (openstack-CLI-compatible format) for future runs.
pub fn run_setup_wizard() -> std::io::Result<CloudAuth> {
    println!("stackboard setup — no OpenStack credentials found.\n");

    let auth_url = prompt("Auth URL (e.g. https://openstack.example.com:5000/v3)")?;
    let username = prompt("Username")?;
    let password = rpassword::prompt_password("Password: ")?;
    let project_name = prompt("Project name")?;
    let user_domain_name = prompt_default("User domain", "Default")?;
    let project_domain_name = prompt_default("Project domain", "Default")?;
    let region_input = prompt_default("Region (blank for default)", "")?;
    let region_name = if region_input.is_empty() {
        None
    } else {
        Some(region_input)
    };

    let auth = CloudAuth {
        auth_url,
        username,
        password,
        project_name,
        user_domain_name,
        project_domain_name,
        region_name,
    };

    if let Some(path) = stackboard_config_path() {
        save_clouds_yaml(&path, &auth)?;
        println!(
            "\nSaved to {} — future runs will pick this up automatically.",
            path.display()
        );
    }

    Ok(auth)
}

fn save_clouds_yaml(path: &PathBuf, auth: &CloudAuth) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let doc = serde_json::json!({
        "clouds": {
            "stackboard": {
                "auth": {
                    "auth_url": auth.auth_url,
                    "username": auth.username,
                    "password": auth.password,
                    "project_name": auth.project_name,
                    "user_domain_name": auth.user_domain_name,
                    "project_domain_name": auth.project_domain_name,
                },
                "region_name": auth.region_name,
            }
        }
    });
    let yaml = serde_yaml::to_string(&doc).map_err(std::io::Error::other)?;
    std::fs::write(path, yaml)?;

    // Credentials are in this file in plaintext (same convention as the
    // openstack CLI's own clouds.yaml) — keep it readable only by the
    // owner.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

fn prompt(label: &str) -> std::io::Result<String> {
    print!("{label}: ");
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    Ok(line.trim().to_string())
}

fn prompt_default(label: &str, default: &str) -> std::io::Result<String> {
    let suffix = if default.is_empty() {
        String::new()
    } else {
        format!(" [{default}]")
    };
    print!("{label}{suffix}: ");
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    let trimmed = line.trim();
    Ok(if trimmed.is_empty() {
        default.to_string()
    } else {
        trimmed.to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clouds_yaml_parses_a_minimal_entry() {
        let yaml = r#"
clouds:
  test:
    auth:
      auth_url: https://example.com:5000/v3
      username: admin
      password: secret
      project_name: admin
    region_name: RegionOne
"#;
        let file: CloudsFile = serde_yaml::from_str(yaml).unwrap();
        let entry = &file.clouds["test"];
        assert_eq!(entry.auth.auth_url, "https://example.com:5000/v3");
        assert_eq!(entry.auth.user_domain_name, "Default"); // default applied
        assert_eq!(entry.region_name.as_deref(), Some("RegionOne"));
    }
}
