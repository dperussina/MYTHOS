//! MYTHOS Hash and Content Addressing
//!
//! This crate implements SHA-256 hashing and content addressing for MYTHOS.
//! All hashes in MYTHOS v0.2 use SHA-256 (alg=1).
//!
//! # Key Types
//! - `Hash` - Self-describing hash with algorithm ID
//! - ID computation functions (TypeID, ToolID, ReceiptID, etc.)
//!
//! # Critical Rules
//! - ALWAYS canonicalize before hashing
//! - Hash struct contains algorithm ID + digest bytes
//! - Receipt ID excludes fields 1 and 11 from hash computation

mod hash;
mod idempotency;
mod receipt;

pub use hash::{sha256, Hash, HashAlg};
pub use idempotency::compute_idempotency_id;
pub use receipt::{canonical_encode_receipt_for_id, compute_receipt_id, AgentID, Receipt};
