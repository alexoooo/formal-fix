# formal-fix

A Rust workspace for a stock exchange with a FIX interface. The initial executable prints its version.

| Crate | Responsibility |
| --- | --- |
| `matching` | Implements price selection and its Verus specification. This crate will own the order book and apply sequenced commands. |
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

`matching::select_resting_order` takes the incoming side and limit in ticks, plus the opposing orders for one instrument. Verus proves that a returned index identifies a live, crossing order and that no eligible order in the slice has a better price. A buy selects the lowest eligible ask and a sell selects the highest eligible bid. Returning `None` proves that there is no eligible order. The scan accepts orders in any price order.

Among eligible orders tied at the best price, Verus also proves that the selector returns the lowest index in the supplied slice. This gives arrival-time priority when the book presents equal-priced orders in arrival order.

The proof covers a single selection. A future fill path must supply the complete opposing book, execute at the selected order's price, and update quantities before selecting again.

### Running the proof on Windows

From a fresh checkout, install Rust `1.100.0-x86_64-pc-windows-msvc` with `rustup toolchain install 1.100.0-x86_64-pc-windows-msvc --profile minimal`. Download the `x86-win.zip` asset for the pinned Verus release and extract its `verus-x86-win` directory under `.tools` at the repository root. The matching manifest pins the corresponding `vstd` version. [Verus release 0.2026.11.22.1ce6a59](https://github.com/verus-lang/verus/releases/tag/release%2F0.2026.11.22.1ce6a59)

In the RustRover PowerShell terminal at the workspace root, run

```powershell
$env:Path = (Resolve-Path .tools/verus-x86-win).Path + ';' + $env:Path
cargo verus verify -p matching --locked
```
