# formal-fix

A Rust workspace for a stock exchange with a FIX interface. The initial executable prints its version.

| Crate | Responsibility |
| --- | --- |
| `matching` | Owns the order book and applies sequenced commands. Matching specifications and Verus proofs will live here alongside the implementation. |
| `fix` | Owns FIX messages, encoding, decoding, and session state. |
| `exchange` | Runs the venue. This is where FIX requests will become sequenced matching commands and fills will become execution reports. |

The two libraries build independently. This lets the matching core be verified and benchmarked without starting a FIX session.

## RustRover

Open the repository root as a project. The root `Cargo.toml` defines all three workspace members. If the Cargo tool window is empty in an existing project, use **Attach Cargo Project** and select the root manifest.

Run the exchange from the gutter beside `main` in `crates/exchange/src/main.rs`, or use the terminal at the repository root.

```sh
cargo run -p exchange
cargo test --workspace
cargo fmt --all --check
```

## Verification

The initial workspace uses stable Rust. Verus setup and the first checked theorem are future work. The first proof target is venue-local price priority for a continuous limit order book. For each fill, the selected resting order must have the best eligible price in the book immediately before that fill.
