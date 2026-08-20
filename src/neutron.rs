use crate::client::Session;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Network {
    pub id: String,
    pub name: String,
    pub status: String,
    pub external: bool,
    pub shared: bool,
    pub subnet_count: usize,
    pub created: String,
}

#[derive(Debug, Clone)]
pub struct SecurityGroup {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_count: usize,
    pub created: String,
}

pub fn list_networks(session: &Session) -> Result<Vec<Network>, String> {
    let body = session.get("network", "/v2.0/networks")?;
    let parsed: RawNetworkList = serde_json::from_value(body)
        .map_err(|e| format!("unexpected response shape from Neutron: {e}"))?;

    Ok(parsed
        .networks
        .into_iter()
        .map(RawNetwork::into_network)
        .collect())
}

pub fn list_security_groups(session: &Session) -> Result<Vec<SecurityGroup>, String> {
    let body = session.get("network", "/v2.0/security-groups")?;
    let parsed: RawSecurityGroupList = serde_json::from_value(body)
        .map_err(|e| format!("unexpected response shape from Neutron: {e}"))?;

    Ok(parsed
        .security_groups
        .into_iter()
        .map(RawSecurityGroup::into_security_group)
        .collect())
}

#[derive(Debug, Deserialize)]
struct RawNetworkList {
    networks: Vec<RawNetwork>,
}

#[derive(Debug, Deserialize)]
struct RawNetwork {
    id: String,
    #[serde(default)]
    name: Option<String>,
    status: String,
    #[serde(default, rename = "router:external")]
    external: bool,
    #[serde(default)]
    shared: bool,
    #[serde(default)]
    subnets: Vec<String>,
    #[serde(default)]
    created_at: String,
}

impl RawNetwork {
    fn into_network(self) -> Network {
        Network {
            id: self.id,
            name: self
                .name
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| "-".to_string()),
            status: self.status,
            external: self.external,
            shared: self.shared,
            subnet_count: self.subnets.len(),
            created: self.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawSecurityGroupList {
    security_groups: Vec<RawSecurityGroup>,
}

#[derive(Debug, Deserialize)]
struct RawSecurityGroup {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: String,
    #[serde(default)]
    security_group_rules: Vec<serde_json::Value>,
    #[serde(default)]
    created_at: String,
}

impl RawSecurityGroup {
    fn into_security_group(self) -> SecurityGroup {
        SecurityGroup {
            id: self.id,
            name: self
                .name
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| "-".to_string()),
            description: if self.description.is_empty() {
                "-".to_string()
            } else {
                self.description
            },
            rule_count: self.security_group_rules.len(),
            created: self.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_network_maps_external_shared_and_subnet_count() {
        let raw = RawNetwork {
            id: "1".into(),
            name: Some("public".into()),
            status: "ACTIVE".into(),
            external: true,
            shared: false,
            subnets: vec!["a".into(), "b".into()],
            created_at: String::new(),
        };
        let n = raw.into_network();
        assert!(n.external);
        assert!(!n.shared);
        assert_eq!(n.subnet_count, 2);
    }

    #[test]
    fn raw_security_group_counts_rules_and_defaults_description() {
        let raw = RawSecurityGroup {
            id: "1".into(),
            name: Some("default".into()),
            description: String::new(),
            security_group_rules: vec![serde_json::json!({}), serde_json::json!({})],
            created_at: String::new(),
        };
        let sg = raw.into_security_group();
        assert_eq!(sg.rule_count, 2);
        assert_eq!(sg.description, "-");
    }
}
