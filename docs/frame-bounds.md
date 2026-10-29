# RFC 4506 Frame Size Bounds

| Stream Category | Max Uncompressed Frame Size | Enforcement |
| :--- | :---: | :--- |
| `ledger` | `64 KiB` | `split_xdr_frames` |
| `transactions` | `16 MiB` | Bounded reader |
| `results` | `16 MiB` | Bounded reader |
