use stellar_checkpoint_core::ancestry::{verify_contiguous_ancestry, HeaderLink};
use stellar_checkpoint_core::checkpoint::{
    archive_relative_path, checkpoint_for_ledger, is_checkpoint_ledger,
};

fn hex_to_32(hex_str: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    let bytes = hex_str.as_bytes();
    for i in 0..32 {
        let hi = (bytes[i * 2] as char).to_digit(16).unwrap() as u8;
        let lo = (bytes[i * 2 + 1] as char).to_digit(16).unwrap() as u8;
        out[i] = (hi << 4) | lo;
    }
    out
}

fn main() {
    println!("=== Stellar History Archive Checkpoint Calculator ===");
    let ledger = 64_791_359;
    assert!(is_checkpoint_ledger(ledger));
    let cp = checkpoint_for_ledger(ledger);
    println!("Checkpoint: {cp} -> {}", archive_relative_path("ledger", cp));

    println!("\n=== Cross-Checkpoint Boundary Verification (Ledger 63 -> 64) ===");
    // Ledger 63 (tip of Checkpoint 1: 0..63)
    let hash_62 = hex_to_32("1111111111111111111111111111111111111111111111111111111111111111");
    let hash_63 = hex_to_32("2222222222222222222222222222222222222222222222222222222222222222");
    // Ledger 64 (start of Checkpoint 2: 64..127): previous_ledger_hash MUST match hash_63
    let hash_64 = hex_to_32("3333333333333333333333333333333333333333333333333333333333333333");

    let header_63 = HeaderLink {
        ledger_seq: 63,
        header_hash: hash_63,
        previous_ledger_hash: hash_62,
    };

    let header_64 = HeaderLink {
        ledger_seq: 64,
        header_hash: hash_64,
        previous_ledger_hash: hash_63, // Cryptographic link across checkpoint boundary!
    };

    let cp1 = checkpoint_for_ledger(63);
    let cp2 = checkpoint_for_ledger(64);
    println!("Ledger 63 belongs to Checkpoint 1: {cp1} (Path: {})", archive_relative_path("ledger", cp1));
    println!("Ledger 64 belongs to Checkpoint 2: {cp2} (Path: {})", archive_relative_path("ledger", cp2));

    match verify_contiguous_ancestry(&[header_63, header_64]) {
        Ok((start, end)) => {
            println!(
                "SUCCESS: Verified contiguous cross-checkpoint hash ancestry from ledger {start} to {end}."
            );
        }
        Err(err) => {
            eprintln!("FAILED: Cross-checkpoint verification failed: {err:?}");
            std::process::exit(1);
        }
    }
}
