use sha2::{Digest, Sha256};

pub fn asset_id(source: &[u8]) -> String {
    format!("sha256:{}", hex_sha256(source))
}

pub fn section_id(asset_id: &str, ordinal: usize) -> String {
    format!("{asset_id}/s{ordinal:04}")
}

fn hex_sha256(input: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let bytes = hasher.finalize();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_deterministic() {
        let id = asset_id(b"Project Alpha memo");
        assert_eq!(id, asset_id(b"Project Alpha memo"));
        assert!(id.starts_with("sha256:"));
        assert_eq!(section_id(&id, 7), format!("{id}/s0007"));
    }
}
