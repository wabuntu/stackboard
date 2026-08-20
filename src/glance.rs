use crate::client::Session;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Image {
    pub id: String,
    pub name: String,
    pub status: String,
    pub disk_format: String,
    pub size_mb: u64,
    pub visibility: String,
    pub created: String,
}

pub fn list_images(session: &Session) -> Result<Vec<Image>, String> {
    let body = session.get("image", "/v2/images")?;
    let parsed: RawImageList = serde_json::from_value(body)
        .map_err(|e| format!("unexpected response shape from Glance: {e}"))?;

    Ok(parsed
        .images
        .into_iter()
        .map(RawImage::into_image)
        .collect())
}

#[derive(Debug, Deserialize)]
struct RawImageList {
    images: Vec<RawImage>,
}

#[derive(Debug, Deserialize)]
struct RawImage {
    id: String,
    #[serde(default)]
    name: Option<String>,
    status: String,
    #[serde(default)]
    disk_format: Option<String>,
    #[serde(default)]
    size: Option<u64>,
    #[serde(default)]
    visibility: Option<String>,
    #[serde(default)]
    created_at: String,
}

impl RawImage {
    fn into_image(self) -> Image {
        Image {
            id: self.id,
            name: self
                .name
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| "-".to_string()),
            status: self.status,
            disk_format: self.disk_format.unwrap_or_else(|| "-".to_string()),
            size_mb: self.size.unwrap_or(0) / (1024 * 1024),
            visibility: self.visibility.unwrap_or_else(|| "-".to_string()),
            created: self.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_image_converts_bytes_to_megabytes() {
        let raw = RawImage {
            id: "1".into(),
            name: Some("cirros".into()),
            status: "active".into(),
            disk_format: Some("qcow2".into()),
            size: Some(21_692_416),
            visibility: Some("public".into()),
            created_at: String::new(),
        };
        assert_eq!(raw.into_image().size_mb, 20);
    }

    #[test]
    fn raw_image_handles_missing_size_and_name() {
        let raw = RawImage {
            id: "1".into(),
            name: None,
            status: "queued".into(),
            disk_format: None,
            size: None,
            visibility: None,
            created_at: String::new(),
        };
        let img = raw.into_image();
        assert_eq!(img.name, "-");
        assert_eq!(img.size_mb, 0);
        assert_eq!(img.disk_format, "-");
    }
}
