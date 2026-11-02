pub mod ancestry;
pub mod checkpoint;
pub mod frame;

pub use ancestry::{
    scan_sequence_gaps, verify_contiguous_ancestry, AncestryError, HeaderLink, LedgerHeaderFrame,
    SequenceGap,
};

#[cfg(test)]
mod tests {
    use super::ancestry::{
        scan_sequence_gaps, verify_contiguous_ancestry, HeaderLink, LedgerHeaderFrame, SequenceGap,
    };
    use super::checkpoint::{archive_relative_path, checkpoint_for_ledger, is_checkpoint_ledger};
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
    fn test_scan_sequence_gaps_contiguous_single_multi_and_out_of_order() {
        fn dummy_frame(seq: u32) -> LedgerHeaderFrame {
            LedgerHeaderFrame {
                ledger_seq: seq,
                header_hash: [0u8; 32],
                previous_ledger_hash: [0u8; 32],
            }
        }

        // 1. Contiguous sequence: [100, 101, 102] -> empty gaps
        let contiguous = vec![dummy_frame(100), dummy_frame(101), dummy_frame(102)];
        assert_eq!(scan_sequence_gaps(&contiguous), vec![]);

        // 2. Single-ledger gap: [100, 102] -> gap [101, 101]
        let single_gap = vec![dummy_frame(100), dummy_frame(102)];
        assert_eq!(
            scan_sequence_gaps(&single_gap),
            vec![SequenceGap {
                expected_start: 101,
                expected_end: 101,
            }]
        );

        // 3. Multi-ledger gap: [100, 105] -> gap [101, 104]
        let multi_gap = vec![dummy_frame(100), dummy_frame(105)];
        assert_eq!(
            scan_sequence_gaps(&multi_gap),
            vec![SequenceGap {
                expected_start: 101,
                expected_end: 104,
            }]
        );

        // 4. Out-of-order frames with multiple gaps: [106, 100, 103, 101]
        // Sorted: 100, 101, 103, 106 -> gaps [102, 102], [104, 105]
        let out_of_order = vec![
            dummy_frame(106),
            dummy_frame(100),
            dummy_frame(103),
            dummy_frame(101),
        ];
        assert_eq!(
            scan_sequence_gaps(&out_of_order),
            vec![
                SequenceGap {
                    expected_start: 102,
                    expected_end: 102,
                },
                SequenceGap {
                    expected_start: 104,
                    expected_end: 105,
                },
            ]
        );

        // Empty and single-element inputs
        assert_eq!(scan_sequence_gaps(&[]), vec![]);
        assert_eq!(scan_sequence_gaps(&[dummy_frame(50)]), vec![]);
    }
}
