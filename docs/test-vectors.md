# Stellar History Archive Checkpoint Test Vectors

## Mainnet Checkpoint Test Vectors

| Network | Checkpoint Sequence | Hex Identifier | Canonical Archive Path |
| :--- | :---: | :---: | :--- |
| Mainnet | `64,791,295` | `03dca2ff` | `ledger/03/dc/a2/ledger-03dca2ff.xdr.gz` |
| Mainnet | `64,791,359` | `03dca33f` | `ledger/03/dc/a3/ledger-03dca33f.xdr.gz` |
| Testnet | `5,052,927` | `004d19ff` | `ledger/00/4d/19/ledger-004d19ff.xdr.gz` |

---

## Cross-Checkpoint Boundary Test Vectors (Ledger 63 -> 64)

In Stellar History Archives, ledger `63` is the terminal ledger of checkpoint `63` (`0..63`), and ledger `64` is the initial ledger of checkpoint `127` (`64..127`). The cryptographic link is established when `header[64].previous_ledger_hash == header[63].header_hash`.

### Test Vector Table

| Field | Ledger 63 (End of Checkpoint 1) | Ledger 64 (Start of Checkpoint 2) |
|---|---|---|
| **Checkpoint Sequence** | `63` (`0x0000003f`) | `127` (`0x0000007f`) |
| **Archive Relative Path** | `ledger/00/00/00/ledger-0000003f.xdr.gz` | `ledger/00/00/00/ledger-0000007f.xdr.gz` |
| **`ledger_seq`** | `63` | `64` |
| **`previous_ledger_hash`** | `1111111111111111111111111111111111111111111111111111111111111111` | `2222222222222222222222222222222222222222222222222222222222222222` |
| **`header_hash`** | `2222222222222222222222222222222222222222222222222222222222222222` | `3333333333333333333333333333333333333333333333333333333333333333` |
| **Link Integrity** | Root anchor for ledger 64 | `previous_ledger_hash` matches Ledger 63's `header_hash` |

---

## CLI Status & Exit Codes Reference

| Exit Code | Status | Meaning |
|:---:|:---|:---|
| `0` | `SUCCESS` | Hash chain verified, checkpoints aligned, and checksum valid. |
| `1` | `USAGE_ERROR` | Missing or invalid command-line invocation syntax. |
| `2` | `ARGUMENT_ERROR` | Invalid ledger range, non-hex payload, or parse failure. |
| `3` | `HASH_MISMATCH` | Cryptographic hash discontinuity or sequence gap detected. |
| `4` | `BUDGET_EXCEEDED`| WAL file or resource limit budget exceeded during preflight inspection. |
