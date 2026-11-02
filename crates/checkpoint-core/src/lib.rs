pub mod ancestry;
pub mod checkpoint;
pub mod frame;

pub use checkpoint::{
    archive_relative_path, checkpoint_for_ledger, is_checkpoint_ledger, latest_published_checkpoint,
    CheckpointError, HistoryArchiveStateSummary,
};

#[cfg(test)]
mod tests {
    use super::ancestry::{verify_contiguous_ancestry, HeaderLink};
    use super::checkpoint::{
        archive_relative_path, checkpoint_for_ledger, is_checkpoint_ledger,
        latest_published_checkpoint, CheckpointError, HistoryArchiveStateSummary,
    };
    use super::frame::split_xdr_frames;

    #[test]
    fn checkpoint_boundaries_match_stellar_archive_spec() {
        assert!(!is_checkpoint_ledger(0));
        assert!(is_checkpoint_ledger(63));
        assert!(!is_checkpoint_ledger(64));
        assert!(is_checkpoint_ledger(64_791_359));
        assert_eq!(checkpoint_for_ledger(64_791_300), 64_791_359);
        assert_eq!(
            archive_relative_path("ledger", 0x03dca33f),
            "ledger/03/dc/a3/ledger-03dca33f.xdr.gz"
        );
    }

    #[test]
    fn rfc4506_frame_splitter_extracts_records() {
        let payload = [0x80, 0x00, 0x00, 0x03, b'x', b'd', b'r'];
        let frames = split_xdr_frames(&payload).expect("valid frame");
        assert_eq!(frames, vec![&b"xdr"[..]]);
    }

    #[test]
    fn ancestry_verifier_validates_hash_links() {
        let h1 = HeaderLink {
            ledger_seq: 100,
            header_hash: [1u8; 32],
            previous_ledger_hash: [0u8; 32],
        };
        let h2 = HeaderLink {
            ledger_seq: 101,
            header_hash: [2u8; 32],
            previous_ledger_hash: [1u8; 32],
        };
        assert_eq!(verify_contiguous_ancestry(&[h1, h2]), Ok((100, 101)));
    }

    #[test]
    fn test_has_parser_and_latest_published_checkpoint_fixtures() {
        // Mainnet HAS fixture
        let mainnet_json = r#"{
            "version": 1,
            "server": "stellar-core 21.0.0",
            "currentLedger": 52631551,
            "networkPassphrase": "Public Global Stellar Network ; September 2015",
            "currentBuckets": []
        }"#;
        let summary = HistoryArchiveStateSummary::parse_json(mainnet_json).expect("valid mainnet HAS");
        assert_eq!(summary.version, 1);
        assert_eq!(summary.current_ledger, 52631551);
        assert_eq!(
            summary.network_passphrase.as_deref(),
            Some("Public Global Stellar Network ; September 2015")
        );
        let chk = summary.latest_checkpoint().expect("valid checkpoint");
        assert_eq!(chk, 52631551); // 52631551 is 64*822368 - 1 (exact checkpoint)

        // Testnet HAS fixture with mid-checkpoint currentLedger
        let testnet_json = r#"{
            "version": 2,
            "currentLedger": 100,
            "networkPassphrase": "Test SDF Network ; September 2015"
        }"#;
        let testnet_summary = HistoryArchiveStateSummary::parse_json(testnet_json).expect("valid testnet HAS");
        assert_eq!(testnet_summary.latest_checkpoint(), Ok(63));

        // Exact boundary at 63
        assert_eq!(latest_published_checkpoint(63), Ok(63));
        assert_eq!(latest_published_checkpoint(64), Ok(63));
        assert_eq!(latest_published_checkpoint(127), Ok(127));
        assert_eq!(latest_published_checkpoint(128), Ok(127));

        // Before first checkpoint (< 63)
        assert_eq!(
            latest_published_checkpoint(62),
            Err(CheckpointError::NoCheckpointPriorToGenesis { ledger_seq: 62 })
        );
        assert_eq!(
            latest_published_checkpoint(0),
            Err(CheckpointError::NoCheckpointPriorToGenesis { ledger_seq: 0 })
        );
    }
}
