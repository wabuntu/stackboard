use crate::client::Session;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Volume {
    pub id: String,
    pub name: String,
    pub status: String,
    pub size_gb: u32,
    pub volume_type: String,
    pub attached_to: Option<String>,
    pub created: String,
}

pub fn delete_volume(session: &Session, id: &str) -> Result<(), String> {
    session.delete("block-storage", &format!("/volumes/{id}"))
}

pub fn list_volumes(session: &Session) -> Result<Vec<Volume>, String> {
    let body = session.get("block-storage", "/volumes/detail")?;
    let parsed: RawVolumeList = serde_json::from_value(body)
        .map_err(|e| format!("unexpected response shape from Cinder: {e}"))?;

    Ok(parsed
        .volumes
        .into_iter()
        .map(RawVolume::into_volume)
        .collect())
}

#[derive(Debug, Deserialize)]
struct RawVolumeList {
    volumes: Vec<RawVolume>,
}

#[derive(Debug, Deserialize)]
struct RawVolume {
    id: String,
    #[serde(default)]
    name: Option<String>,
    status: String,
    size: u32,
    #[serde(default)]
    volume_type: Option<String>,
    #[serde(default)]
    attachments: Vec<RawAttachment>,
    #[serde(default)]
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct RawAttachment {
    server_id: String,
    device: String,
}

impl RawVolume {
    fn into_volume(self) -> Volume {
        let attached_to = self.attachments.first().map(|a| {
            let short_id: String = a.server_id.chars().take(8).collect();
            format!("{short_id} ({})", a.device)
        });

        Volume {
            id: self.id,
            name: self
                .name
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| "-".to_string()),
            status: self.status,
            size_gb: self.size,
            volume_type: self.volume_type.unwrap_or_else(|| "-".to_string()),
            attached_to,
            created: self.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_volume_uses_dash_for_missing_name() {
        let raw = RawVolume {
            id: "1".into(),
            name: None,
            status: "available".into(),
            size: 5,
            volume_type: None,
            attachments: vec![],
            created_at: String::new(),
        };
        let v = raw.into_volume();
        assert_eq!(v.name, "-");
        assert_eq!(v.volume_type, "-");
        assert_eq!(v.attached_to, None);
    }

    #[test]
    fn raw_volume_maps_name_and_attachment() {
        let raw = RawVolume {
            id: "fb81f299-d42a-43ec-acef-bc85060ab6c8".into(),
            name: Some("test-vol-01".into()),
            status: "in-use".into(),
            size: 1,
            volume_type: Some("lvmdriver-1".into()),
            attachments: vec![RawAttachment {
                server_id: "2c05a618-a179-4e7d-b98b-f3975a40035e".into(),
                device: "/dev/vdb".into(),
            }],
            created_at: "2026-08-20T16:48:57.000000".into(),
        };
        let v = raw.into_volume();
        assert_eq!(v.name, "test-vol-01");
        assert_eq!(v.attached_to.as_deref(), Some("2c05a618 (/dev/vdb)"));
    }

    #[test]
    fn raw_volume_treats_empty_name_as_dash() {
        let raw = RawVolume {
            id: "1".into(),
            name: Some(String::new()),
            status: "available".into(),
            size: 1,
            volume_type: None,
            attachments: vec![],
            created_at: String::new(),
        };
        assert_eq!(raw.into_volume().name, "-");
    }
}
