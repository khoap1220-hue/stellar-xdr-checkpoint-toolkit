//! Stellar History Archive checkpoint boundary calculations.
//! Checkpoints occur every 64 ledgers at sequences `64*k - 1` (`63, 127, 191, ...`).

use std::fmt;

/// Structured error type for checkpoint alignment and history archive state errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckpointError {
    NoCheckpointPriorToGenesis { ledger_seq: u32 },
    InvalidHistoryArchiveState(String),
}

impl fmt::Display for CheckpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckpointError::NoCheckpointPriorToGenesis { ledger_seq } => write!(
                f,
                "ledger sequence {ledger_seq} precedes first published checkpoint at sequence 63"
            ),
            CheckpointError::InvalidHistoryArchiveState(msg) => {
                write!(f, "invalid history archive state JSON: {msg}")
            }
        }
    }
}

impl std::error::Error for CheckpointError {}

/// Summary representation of a Stellar `.well-known/stellar-history.json` Root History Archive State (HAS).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryArchiveStateSummary {
    pub version: u32,
    pub current_ledger: u32,
    pub network_passphrase: Option<String>,
}

fn parse_json_u32(raw: &str, key: &str) -> Option<u32> {
    let pattern = format!("\"{}\":", key);
    let pos = raw.find(&pattern)?;
    let rest = raw[pos + pattern.len()..].trim_start();
    let mut end = 0;
    for c in rest.chars() {
        if c.is_ascii_digit() {
            end += 1;
        } else {
            break;
        }
    }
    if end > 0 {
        rest[..end].parse().ok()
    } else {
        None
    }
}

fn parse_json_str(raw: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":", key);
    let pos = raw.find(&pattern)?;
    let rest = raw[pos + pattern.len()..].trim_start();
    if rest.starts_with('"') {
        let s = &rest[1..];
        let end = s.find('"')?;
        Some(s[..end].to_string())
    } else {
        None
    }
}

impl HistoryArchiveStateSummary {
    /// Parses root HAS JSON content into a typed summary.
    pub fn parse_json(json_str: &str) -> Result<Self, CheckpointError> {
        let version = parse_json_u32(json_str, "version")
            .ok_or_else(|| CheckpointError::InvalidHistoryArchiveState("missing 'version'".into()))?;
        let current_ledger = parse_json_u32(json_str, "currentLedger")
            .ok_or_else(|| CheckpointError::InvalidHistoryArchiveState("missing 'currentLedger'".into()))?;
        let network_passphrase = parse_json_str(json_str, "networkPassphrase");

        Ok(Self {
            version,
            current_ledger,
            network_passphrase,
        })
    }

    /// Resolves the latest completed checkpoint for this HAS file.
    pub fn latest_checkpoint(&self) -> Result<u32, CheckpointError> {
        latest_published_checkpoint(self.current_ledger)
    }
}

/// Resolves the latest completed `64*k - 1` checkpoint ledger for any `current_ledger >= 63`.
///
/// Returns `CheckpointError::NoCheckpointPriorToGenesis` if `current_ledger < 63`.
pub fn latest_published_checkpoint(current_ledger: u32) -> Result<u32, CheckpointError> {
    if current_ledger < 63 {
        return Err(CheckpointError::NoCheckpointPriorToGenesis {
            ledger_seq: current_ledger,
        });
    }
    let aligned = ((current_ledger as u64 + 1) / 64) * 64 - 1;
    Ok(aligned as u32)
}

/// Returns `true` if `ledger_seq` is a valid Stellar History Archive checkpoint ledger.
#[inline]
pub const fn is_checkpoint_ledger(ledger_seq: u32) -> bool {
    ledger_seq >= 63 && ((ledger_seq as u64 + 1) % 64 == 0)
}

/// Returns the Stellar History Archive checkpoint sequence containing `ledger_seq`.
#[inline]
pub const fn checkpoint_for_ledger(ledger_seq: u32) -> u32 {
    let aligned = ((ledger_seq as u64 + 64) / 64) * 64 - 1;
    aligned as u32
}

/// Formats the canonical History Archive relative path for a category and checkpoint.
pub fn archive_relative_path(category: &str, checkpoint: u32) -> String {
    let b0 = (checkpoint >> 24) & 0xff;
    let b1 = (checkpoint >> 16) & 0xff;
    let b2 = (checkpoint >> 8) & 0xff;
    format!(
        "{category}/{b0:02x}/{b1:02x}/{b2:02x}/{category}-{checkpoint:08x}.xdr.gz"
    )
}
