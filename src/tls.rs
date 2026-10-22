use std::path::Path;

/// Reads a PEM file with one or more CA certificates (for `--cacert`,
/// `OS_CACERT`, or `cacert` in clouds.yaml). These certificates are
/// trusted in addition to the operating system's own trust store.
pub fn load_ca_bundle(path: &Path) -> Result<Vec<reqwest::Certificate>, String> {
    let bytes = std::fs::read(path)
        .map_err(|e| format!("could not read CA file {}: {e}", path.display()))?;
    let certs = reqwest::Certificate::from_pem_bundle(&bytes).map_err(|e| {
        format!(
            "CA file {} is not a valid PEM certificate file: {e}",
            path.display()
        )
    })?;
    if certs.is_empty() {
        return Err(format!(
            "CA file {} has no PEM certificates (expected one or more \
             \"-----BEGIN CERTIFICATE-----\" blocks)",
            path.display()
        ));
    }
    Ok(certs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_data(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data")
            .join(name)
    }

    #[test]
    fn loads_every_certificate_in_a_bundle() {
        let certs = load_ca_bundle(&test_data("two-cas.pem")).unwrap();
        assert_eq!(certs.len(), 2);
    }

    #[test]
    fn empty_file_is_an_error() {
        let err = load_ca_bundle(&test_data("empty.pem")).unwrap_err();
        assert!(err.contains("no PEM certificates"), "{err}");
    }

    #[test]
    fn missing_file_is_an_error() {
        let err = load_ca_bundle(&test_data("does-not-exist.pem")).unwrap_err();
        assert!(err.contains("could not read CA file"), "{err}");
    }
}
