use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackManifest {
    pub pack: String,

    #[serde(default)]
    pub version: String,

    #[serde(default)]
    pub vectors: Vec<VectorEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEntry {
    /// Vector ID in format "{SUITE}_{NUMBER}", e.g. "CAN_001"
    pub id: String,

    /// Human-readable description
    pub description: String,

    /// RFC reference
    pub rfc_reference: String,

    /// File paths (keys vary by vector type)
    /// CAN: bin, json
    /// MERKLE: input_json, leaf_bin, rootcid_hex
    /// BLOB: chunks_json, payload_bin, rootcid_hex, rootnode_bin
    pub files: HashMap<String, String>,

    /// Expected outputs (keys vary by vector type)
    /// CAN: sha256_of_bin
    /// MERKLE: root_cid
    /// RECEIPT: receipt_id
    pub expected: HashMap<String, serde_json::Value>,
}

impl PackManifest {
    pub fn load(pack_dir: &Path) -> Result<Self> {
        let manifest_path = pack_dir.join("manifest.json");
        let contents = fs::read_to_string(&manifest_path)
            .with_context(|| format!("Failed to read manifest at {:?}", manifest_path))?;

        let manifest: PackManifest =
            serde_json::from_str(&contents).with_context(|| "Failed to parse manifest.json")?;

        Ok(manifest)
    }

    /// Get vectors for a specific suite
    pub fn vectors_for_suite(&self, suite: &str) -> Vec<&VectorEntry> {
        use ctvp_runner::suite::prefix_for_suite;

        if suite == "all" {
            return self.vectors.iter().collect();
        }

        let prefix = prefix_for_suite(suite);
        if prefix.is_empty() {
            return vec![];
        }

        self.vectors
            .iter()
            .filter(|v| v.id.starts_with(prefix))
            .collect()
    }

    /// Get a specific vector by ID
    pub fn get_vector(&self, id: &str) -> Option<&VectorEntry> {
        self.vectors.iter().find(|v| v.id == id)
    }
}

impl VectorEntry {
    /// Resolve a file path relative to pack directory
    pub fn resolve_file(&self, pack_dir: &Path, key: &str) -> Option<PathBuf> {
        self.files.get(key).map(|p| pack_dir.join(p))
    }

    /// Get the bin file path
    pub fn bin_path(&self, pack_dir: &Path) -> Option<PathBuf> {
        self.resolve_file(pack_dir, "bin")
    }

    /// Get expected SHA256 from manifest (CAN vectors)
    pub fn expected_sha256(&self) -> Option<String> {
        self.expected
            .get("sha256_of_bin")
            .and_then(|v| v.as_str())
            .map(String::from)
    }
}
