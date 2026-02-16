# eStream Platform Feedback from PolyKit Development

> Running document capturing what works, what doesn't, and upstream improvement suggestions
> as we build PolyKit on the eStream v0.8.1 platform.
>
> This will be reviewed periodically and translated into `polyquantum/estream-io` issues.

---

## What Works Well

### ESCIR WASM Client Build Pipeline (#550)
- The `estream-dev build-wasm-client` single-command build is excellent DX — one command from ESCIR to signed WASM
- Size budgets (128 KB/circuit, 512 KB total) are well-calibrated for individual app circuits
- `.escd` package format is clean and self-contained
- ML-DSA-87 signing with detached `.wasm.sig` is the right pattern for integrity

### ABI Contract
- Required `evaluate(i32) -> i32` is simple and correct
- Host imports (`estream::sha3_256`, `estream::get_random`, etc.) cover the crypto primitives well
- Memory budget (64 pages / 4 MB) is reasonable for client-side circuits

### ESCIR Circuit Definitions
- YAML format is readable and expressive
- StreamSight annotations (`@streamsight_filter`, `@streamsight_sensitivity`) are well-designed
- `emissions` + `exports` structure maps cleanly to the WASM ABI

---

## Pain Points / Gaps Discovered

### 1. Size Budget for Shared Libraries (OBSERVATION)
- **Issue**: PolyKit is a *shared* framework, not a single app. The 128 KB/circuit and 512 KB total budgets were designed for individual app circuits.
- **Question**: When polykit-core, polykit-eslite, polykit-console, and polykit-sanitize are combined into a single WASM module, will we hit the 512 KB ceiling? Shared library overhead (PQ crypto primitives, ESLite engine, sanitization regexes) may be significant.
- **Suggestion**: Consider a "shared library" exemption or a two-tier budget: ≤512 KB for shared framework WASM, ≤512 KB for app-specific circuits, with a combined ceiling of ≤1 MB.
- **Status**: Needs measurement once integration is complete.

### 2. Host Import Gaps (OBSERVATION)
- **Issue**: The current allowed host imports (`estream::sha3_256`, `estream::ctx_*`, `estream::get_random`, `estream::get_time`, `estream::log_debug`) may not cover all shared framework needs.
- **Potentially needed**:
  - `estream::eslite_query` — execute ESLite queries from WASM
  - `estream::eslite_mutate` — write to ESLite from WASM
  - `estream::wire_subscribe` — manage wire protocol subscriptions from WASM
  - `estream::wire_emit` — emit to lex topics from WASM
  - `estream::ml_dsa_87_sign` / `estream::ml_dsa_87_verify` — if crypto stays in host
  - `estream::ml_kem_1024_encapsulate` / `estream::ml_kem_1024_decapsulate`
- **Question**: Should PQ crypto run inside WASM (via compiled-in Rust crates) or be delegated to host imports? Host imports would reduce WASM binary size but add ABI surface.
- **Status**: Needs architectural decision.

### 3. Circuit Composition / Imports (QUESTION)
- **Issue**: ESCIR circuits currently seem designed as standalone units. PolyKit needs circuits that *compose* — e.g., `polymail-inbox` should be able to `import polykit-identity` and `polykit-metering`.
- **Question**: Does the ESCIR `imports` mechanism support cross-circuit dependencies at the WASM level? Or does composition happen at the Rust crate level before WASM compilation?
- **Suggestion**: If not already supported, add an ESCIR `imports:` directive that resolves to Rust `use` statements in the codegen output.
- **Status**: Needs investigation.

### 4. Widget Data Pipeline Host Interface (GAP)
- **Issue**: Console widgets need to receive stream data in WASM, process it, and return render-ready JSON to the TS layer. The current ABI (`evaluate(i32) -> i32`) is designed for single-invocation circuits, not continuous data pipelines.
- **Suggestion**: Consider a streaming variant of the ABI: `on_stream_data(topic_ptr, data_ptr, len) -> (output_ptr, output_len)` that the host calls when new stream data arrives, and the WASM returns processed widget data.
- **Status**: Needs discussion.

### 5. Demo Mode / ESZ Fixtures in WASM (MINOR)
- **Issue**: ESZ test fixtures need to be loaded into WASM for demo mode. The current test framework (`wasm_test.rs`) runs fixtures *against* WASM from the host side. For demo mode, fixtures need to be loaded *into* WASM as mock stream data.
- **Suggestion**: Add an `estream::load_fixture(fixture_ptr, len)` host import for demo/test mode.
- **Status**: Minor — can be worked around by passing fixtures via `init_app()`.

---

## Upstream Feature Requests

### FR-1: Streaming ABI Extension
Add an optional streaming ABI alongside the batch `evaluate()`:
```
// Host calls this when new stream data arrives for a subscribed topic
fn on_stream_data(topic_ptr: i32, topic_len: i32, data_ptr: i32, data_len: i32) -> i32
```

### FR-2: ESLite Host Imports
Add ESLite read/write host imports so WASM circuits can query and mutate ESLite directly:
```
estream::eslite_query(table_ptr, table_len, filter_ptr, filter_len) -> (result_ptr, result_len)
estream::eslite_insert(table_ptr, table_len, data_ptr, data_len) -> status
estream::eslite_update(table_ptr, table_len, key_ptr, key_len, data_ptr, data_len) -> status
estream::eslite_delete(table_ptr, table_len, key_ptr, key_len) -> status
```

### FR-3: Wire Protocol Host Imports
Add wire protocol host imports for WASM-managed subscriptions:
```
estream::wire_subscribe(topic_ptr, topic_len) -> subscription_id
estream::wire_unsubscribe(subscription_id) -> status
estream::wire_emit(topic_ptr, topic_len, data_ptr, data_len) -> status
```

### FR-4: Shared Library Size Budget Tier
Support a higher size budget for shared framework WASM modules (≤1 MB) vs app-specific circuits (≤512 KB).

---

## Codegen Quality Observations

*To be filled in once ESCIR → Rust codegen is exercised against PolyKit circuits.*

---

## Testing Observations

*To be filled in once ESF/ESZ test vectors are generated for PolyKit circuits.*
