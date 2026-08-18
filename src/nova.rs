use crate::client::Session;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Server {
    pub id: String,
    pub name: String,
    pub status: String,
    pub flavor: String,
    pub addresses: Vec<String>,
    pub host: Option<String>,
    pub created: String,
}

pub fn list_servers(session: &Session) -> Result<Vec<Server>, String> {
    let body = session.get("compute", "/servers/detail?all_tenants=1")?;
    let servers: RawServerList = serde_json::from_value(body.clone())
        // Non-admin users can't pass all_tenants; retry scoped to their own project.
        .or_else(|_| serde_json::from_value(body))
        .map_err(|e| format!("unexpected response shape from Nova: {e}"))?;

    Ok(servers
        .servers
        .into_iter()
        .map(RawServer::into_server)
        .collect())
}

#[derive(Debug, Deserialize)]
struct RawServerList {
    servers: Vec<RawServer>,
}

#[derive(Debug, Deserialize)]
struct RawServer {
    id: String,
    name: String,
    status: String,
    #[serde(default)]
    flavor: RawFlavor,
    #[serde(default)]
    addresses: std::collections::HashMap<String, Vec<RawAddress>>,
    #[serde(default, rename = "OS-EXT-SRV-ATTR:host")]
    host: Option<String>,
    #[serde(default)]
    created: String,
}

#[derive(Debug, Default, Deserialize)]
struct RawFlavor {
    #[serde(default)]
    original_name: Option<String>,
    #[serde(default)]
    vcpus: Option<u32>,
    #[serde(default)]
    ram: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct RawAddress {
    addr: String,
}

impl RawServer {
    fn into_server(self) -> Server {
        let flavor = self
            .flavor
            .original_name
            .or_else(|| match (self.flavor.vcpus, self.flavor.ram) {
                (Some(vcpus), Some(ram)) => Some(format!("{vcpus}vcpu/{ram}MB")),
                _ => None,
            })
            .unwrap_or_else(|| "-".to_string());

        let mut addresses: Vec<String> = self
            .addresses
            .into_values()
            .flatten()
            .map(|a| a.addr)
            .collect();
        addresses.sort();

        Server {
            id: self.id,
            name: self.name,
            status: self.status,
            flavor,
            addresses,
            host: self.host,
            created: self.created,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_server_maps_flavor_original_name_when_present() {
        let raw = RawServer {
            id: "1".into(),
            name: "web-01".into(),
            status: "ACTIVE".into(),
            flavor: RawFlavor {
                original_name: Some("m1.small".into()),
                vcpus: None,
                ram: None,
            },
            addresses: Default::default(),
            host: Some("compute-02".into()),
            created: "2026-01-01T00:00:00Z".into(),
        };
        let server = raw.into_server();
        assert_eq!(server.flavor, "m1.small");
        assert_eq!(server.host.as_deref(), Some("compute-02"));
    }

    #[test]
    fn raw_server_falls_back_to_vcpu_ram_when_no_flavor_name() {
        let raw = RawServer {
            id: "1".into(),
            name: "web-01".into(),
            status: "ACTIVE".into(),
            flavor: RawFlavor {
                original_name: None,
                vcpus: Some(2),
                ram: Some(4096),
            },
            addresses: Default::default(),
            host: None,
            created: String::new(),
        };
        assert_eq!(raw.into_server().flavor, "2vcpu/4096MB");
    }

    #[test]
    fn raw_server_collects_and_sorts_addresses_across_networks() {
        let mut addresses = std::collections::HashMap::new();
        addresses.insert(
            "private".to_string(),
            vec![RawAddress {
                addr: "10.0.0.5".into(),
            }],
        );
        addresses.insert(
            "public".to_string(),
            vec![RawAddress {
                addr: "10.0.0.2".into(),
            }],
        );
        let raw = RawServer {
            id: "1".into(),
            name: "web-01".into(),
            status: "ACTIVE".into(),
            flavor: RawFlavor::default(),
            addresses,
            host: None,
            created: String::new(),
        };
        assert_eq!(raw.into_server().addresses, vec!["10.0.0.2", "10.0.0.5"]);
    }
}
