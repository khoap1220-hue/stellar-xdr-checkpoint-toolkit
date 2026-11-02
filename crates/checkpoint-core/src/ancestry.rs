//! Deterministic verification of contiguous `previous_ledger_hash` links.

/// Decoded representation of a Stellar LedgerHeader link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerHeaderFrame {
    pub ledger_seq: u32,
    pub header_hash: [u8; 32],
    pub previous_ledger_hash: [u8; 32],
}

/// Backward compatibility alias for `LedgerHeaderFrame`.
pub type HeaderLink = LedgerHeaderFrame;

/// Represents an inclusive range of missing ledger sequences `[expected_start, expected_end]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceGap {
    pub expected_start: u32,
    pub expected_end: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AncestryError {
    EmptyChain,
    SequenceGap { expected: u32, found: u32 },
    HashMismatch { at_ledger: u32 },
}

/// Scans a slice of `LedgerHeaderFrame` records and reports all missing sequence intervals.
///
/// Handles out-of-order frames by ordering sequences and deduplicating duplicates.
/// Returns an empty vector if headers are fully contiguous without missing sequences.
pub fn scan_sequence_gaps(headers: &[LedgerHeaderFrame]) -> Vec<SequenceGap> {
    if headers.len() < 2 {
        return Vec::new();
    }

    let mut seqs: Vec<u32> = headers.iter().map(|h| h.ledger_seq).collect();
    seqs.sort_unstable();
    seqs.dedup();

    let mut gaps = Vec::new();
    for window in seqs.windows(2) {
        let prev = window[0];
        let curr = window[1];
        if curr > prev.saturating_add(1) {
            gaps.push(SequenceGap {
                expected_start: prev + 1,
                expected_end: curr - 1,
            });
        }
    }
    gaps
}

/// Verifies that `headers` form an unbroken contiguous hash chain from `first` to `last`.
pub fn verify_contiguous_ancestry(headers: &[HeaderLink]) -> Result<(u32, u32), AncestryError> {
    let first = headers.first().ok_or(AncestryError::EmptyChain)?;
    for window in headers.windows(2) {
        let prev = &window[0];
        let curr = &window[1];
        if curr.ledger_seq != prev.ledger_seq.saturating_add(1) {
            return Err(AncestryError::SequenceGap {
                expected: prev.ledger_seq.saturating_add(1),
                found: curr.ledger_seq,
            });
        }
        if curr.previous_ledger_hash != prev.header_hash {
            return Err(AncestryError::HashMismatch {
                at_ledger: curr.ledger_seq,
            });
        }
    }
    let last = headers.last().expect("non-empty");
    Ok((first.ledger_seq, last.ledger_seq))
}
