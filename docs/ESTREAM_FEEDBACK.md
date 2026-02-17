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

## ESCIR Annotation Enhancement Proposals

> These proposals emerged from writing 15+ circuit definitions across PolyKit,
> Poly Data, and Poly Messenger, plus reviewing circuits in SynergyCarbon,
> TrueResolve, and TakeTitle. Filed as `polyquantum/estream-io` issue.

### Proposal 1: Annotation Profiles (`profile:`)

**Problem**: Every circuit repeats the same 5-7 annotations verbatim. Across a 6-circuit app, that's 30-42 duplicate annotation lines. Copy-paste errors are inevitable, and changing a default requires touching every file.

**Current (verbose, repeated per circuit)**:
```yaml
annotations:
  "@streamsight_filter": "baseline"
  "@streamsight_sensitivity": 2.0
  "@streamsight_warmup": 1000
  "@streamsight_sample_normal": 0.01
  "@precision_class": "C"
  "@witness_tier": 2
  "@alert_on_error": true
  "@memory_tier": "L1"
```

**Proposed**:
```yaml
# One line replaces 8 annotations
profile: "poly-app-standard"

# Override specific values when needed
override:
  "@streamsight_sensitivity": 3.5
```

**Profile Definition Format** (platform-level or app-level):
```yaml
# profiles/poly-app-standard.yaml
profile: "poly-app-standard"
description: "Standard Poly Labs app circuit profile"
inherits: "estream-default"       # optional: inherit from another profile
annotations:
  "@streamsight_filter": "baseline"
  "@streamsight_sensitivity": 2.0
  "@streamsight_warmup": 1000
  "@streamsight_sample_normal": 0.01
  "@precision_class": "C"
  "@witness_tier": 2
  "@alert_on_error": true
  "@memory_tier": "L1"
  "@hardware_required": false
```

**Platform-provided profiles**:
- `"estream-default"` — sensible defaults for any circuit
- `"high-security"` — witness_tier: 4, hardware_required: true, precision_class: A
- `"b2b-integration"` — witness_tier: 3, streamsight_sensitivity: 1.5
- `"embedded-iot"` — memory_tier: L3, precision_class: D, budget.wasm: 32KB

**Codegen impact**: The ESCIR compiler resolves profiles at parse time before codegen. The Rust generator sees fully-expanded annotations — zero codegen changes needed. Profile resolution is a preprocessing step:

```
parse YAML → resolve profile → merge overrides → existing codegen pipeline
```

**Validation**: `estream-dev lint` warns on undefined profiles and detects annotation conflicts between profile and overrides.

---

### Proposal 2: `@budget` — Size and Resource Budgets in Circuit Definition

**Problem**: Size budgets are currently enforced via CLI flags (`--enforce-budget`) with global defaults (128 KB/circuit, 512 KB total). But different circuits have different budgets — a shared framework circuit needs more room than a simple rate limiter. The budget should live in the circuit definition, not in CI scripts.

**Proposed**:
```yaml
annotations:
  "@budget":
    wasm: "64KB"         # max WASM binary size for this circuit
    memory: "2MB"        # max WASM linear memory (pages)
    instantiation: "30ms" # max instantiation time
    bundle_class: "shared" # "app" (default, ≤512KB) or "shared" (≤1MB)
```

**Shorthand for common cases**:
```yaml
annotations:
  "@budget": "compact"   # preset: wasm: 32KB, memory: 1MB, instantiation: 20ms
  "@budget": "standard"  # preset: wasm: 128KB, memory: 4MB, instantiation: 50ms
  "@budget": "framework" # preset: wasm: 256KB, memory: 4MB, instantiation: 75ms
```

**Codegen impact**: The ESCIR compiler emits budget metadata into the `.escd` manifest:

```json
{
  "budget": {
    "wasm_bytes": 65536,
    "memory_pages": 32,
    "instantiation_ms": 30,
    "bundle_class": "shared"
  }
}
```

`estream-dev build-wasm-client` reads budget from the circuit definition (not CLI flags) and enforces per-circuit. The `package-escd` command sums budgets by bundle_class and enforces the aggregate ceiling.

**Codegen: Rust `#[cfg]` gating**:
When `@budget.wasm` is tight (< 64KB), the codegen could emit conditional compilation flags:

```rust
// Auto-generated by ESCIR codegen when @budget.wasm < 64KB
#[cfg(feature = "compact")]
use estream_kernel::crypto::ml_dsa_87_lite; // smaller implementation
#[cfg(not(feature = "compact"))]
use estream_kernel::crypto::ml_dsa_87;
```

---

### Proposal 3: `@meters` — Metering Dimension Declaration

**Problem**: Every Poly app uses the 8-dimension metering model (E/H/B/S/O/P/C/M), but developers must manually add `ctx.metering.record()` calls in every export function. This is easy to forget and hard to audit.

**Proposed**:
```yaml
annotations:
  "@meters": ["B", "S", "C"]  # this circuit consumes bandwidth, storage, circuits
```

**Full form with per-dimension details**:
```yaml
annotations:
  "@meters":
    B:
      per_call: "estimate"   # auto-estimate from payload size
      cap: "10MB"            # per-invocation cap
    S:
      per_call: "explicit"   # developer provides value in export fn
    C:
      per_call: "auto"       # auto-increment on every evaluate() call
```

**Codegen impact — auto-generated metering hooks**:

When `@meters` is declared, the ESCIR → Rust codegen wraps every exported function with metering instrumentation:

```rust
// Without @meters — developer must remember to add metering
pub fn upload_file(ctx: &mut Context, payload: &[u8]) -> Result<()> {
    // developer must manually add:
    ctx.metering.record(Dimension::Bandwidth, payload.len() as u64);
    ctx.metering.record(Dimension::Circuits, 1);
    // ... actual logic
}

// With @meters: ["B", "C"] — codegen auto-generates the wrapper
pub fn upload_file(ctx: &mut Context, payload: &[u8]) -> Result<()> {
    // AUTO-GENERATED by ESCIR codegen from @meters annotation
    ctx.metering.record(Dimension::Bandwidth, payload.len() as u64);
    ctx.metering.record(Dimension::Circuits, 1);
    // --- end auto-generated ---

    // developer's logic only:
    encrypt_and_scatter(ctx, payload)
}
```

For `per_call: "auto"` dimensions, the codegen adds the increment before function body. For `per_call: "estimate"`, it measures input/output sizes. For `per_call: "explicit"`, it generates a `ctx.metering` accessor the developer calls.

**Validation**: `estream-dev lint` warns if a circuit emits to wire topics but doesn't declare `@meters: ["B"]` (bandwidth), or if it writes to ESLite but doesn't declare `@meters: ["S"]` (storage).

---

### Proposal 4: `@sanitize` — Compliance Scope Declaration

**Problem**: Compliance sanitization (PII/PCI/HIPAA/GDPR) is critical but opt-in. A developer can forget to run the sanitization pipeline on an emission, and nothing catches it until an audit. Compliance that can be forgotten isn't compliance.

**Proposed**:
```yaml
annotations:
  "@sanitize": ["HIPAA", "PCI-DSS", "GDPR"]
```

**Behavior**: When `@sanitize` is declared, the runtime automatically applies the 3-stage sanitization pipeline to **all emissions** from this circuit before they cross any trust boundary (WASM → host, wire emit, ESLite write). The circuit developer doesn't call the pipeline — it's injected by the runtime.

**Granular control**:
```yaml
annotations:
  "@sanitize":
    scope: ["HIPAA", "PCI-DSS", "GDPR"]
    emissions:
      - topic: "*.telemetry"
        level: "full"          # all 3 stages
      - topic: "*.metrics.*"
        level: "detect_only"   # stage 1 only — flag but don't redact
    exempt:
      - topic: "*.audit.*"     # audit trail is already sanitized, skip
    on_detection: "redact"     # "redact" (default), "block", "warn"
```

**Codegen impact — emission wrapper injection**:

The ESCIR → Rust codegen wraps every `emit()` call with sanitization:

```rust
// Auto-generated wrapper for emission from @sanitize circuit
fn emit_sanitized(ctx: &mut Context, topic: &str, payload: &[u8]) -> Result<()> {
    // AUTO-GENERATED from @sanitize: ["HIPAA", "PCI-DSS", "GDPR"]
    let sanitized = polykit_sanitize::sanitize_with_scope(
        payload,
        &[Regulation::Hipaa, Regulation::PciDss, Regulation::Gdpr],
    );
    
    // Emit audit trail to witness stream
    ctx.emit(
        &format!("{}.sanitization.audit", ctx.app_namespace()),
        &sanitized.audit_entries,
    )?;
    
    // Emit sanitized payload
    ctx.emit(topic, &sanitized.sanitized_data)
}
```

**Codegen: compile-time exhaustiveness check**:

The codegen can verify at compile time that every emission path in the circuit goes through the sanitization wrapper. If a developer adds a raw `ctx.emit()` that bypasses sanitization, the compiler errors:

```
error[ESCIR-S001]: unsanitized emission in @sanitize circuit
  --> polydata-storage-router/circuit.escir.yaml:42
  |
  | export fn scatter_complete:
  |     ctx.emit("polylabs.data.upload.confirm", raw_payload)
  |     ^^^^^^^^ this emission bypasses @sanitize pipeline
  |
  = help: use ctx.emit_sanitized() or add topic to @sanitize.exempt
```

---

### Proposal 5: `@rbac` — Role Requirements

**Problem**: Widget processors and console circuits need role requirements, but these are currently declared in TypeScript widget registration code, disconnected from the circuit definition. This means RBAC is enforced in the TS layer (which we want to minimize) instead of the WASM layer (where enforcement should live).

**Proposed**:
```yaml
annotations:
  "@rbac":
    roles: ["operator", "compliance"]  # required roles (OR — any satisfies)
    scope: "app"                        # "app" (prefix with app_id) or "global"
```

**Shorthand**:
```yaml
annotations:
  "@rbac": ["operator"]    # defaults to scope: "app"
```

**Codegen impact**: Two things happen:

1. The WASM entry point auto-checks roles before executing any export:

```rust
// AUTO-GENERATED from @rbac: ["operator"]
pub fn evaluate(ctx_ptr: i32) -> i32 {
    let ctx = Context::from_ptr(ctx_ptr);
    
    // RBAC gate — generated from @rbac annotation
    if !ctx.session.has_any_role(&["polydata-operator"]) {
        return ErrorCode::Unauthorized as i32;
    }
    
    // ... actual circuit logic
}
```

2. The `.escd` manifest includes role metadata, so console widget registration can auto-derive `requiredRoles`:

```json
{
  "rbac": {
    "roles": ["operator", "compliance"],
    "scope": "app"
  }
}
```

The `@polykit/react` `WidgetShell` reads this from the manifest — no need to duplicate roles in TS.

---

### Proposal 6: `composes:` — Circuit Composition with Versioning

**Problem**: ESCIR circuits are designed as standalone units, but real apps need composition. `polymail-inbox` depends on `polykit-identity` and `polykit-metering`. Today this dependency is implicit (Rust `use` statements). The build pipeline can't validate, tree-shake, or enforce budgets across the composition graph.

**Proposed**:
```yaml
escir: "0.8.1"
name: polymail-inbox
version: "0.1.0"

composes:
  - circuit: polykit-identity
    version: "^0.1.0"
    imports: [derive_keys, sign, verify]    # selective imports
  - circuit: polykit-metering
    version: "^0.1.0"
    imports: [record_usage, check_limits]
  - circuit: polykit-sanitize
    version: "^0.1.0"
    imports: [sanitize]                     # imports all if omitted
```

**Codegen impact — dependency resolution and Rust `use` generation**:

The ESCIR compiler resolves `composes:` into a dependency graph, validates version constraints, and generates Rust imports:

```rust
// AUTO-GENERATED from composes: section
use polykit_identity::{derive_keys, sign, verify};
use polykit_metering::{record_usage, check_limits};
use polykit_sanitize::sanitize;
```

**Codegen: tree-shaking from selective imports**:

When `imports:` lists specific functions, the codegen emits `#[allow(dead_code)]` only for those functions, and `wasm-opt` eliminates unused code from composed circuits. This directly helps hit size budgets:

```
polykit-identity full:   64 KB
polykit-identity [derive_keys, sign, verify]:  28 KB  (56% smaller)
```

**Codegen: budget aggregation**:

The build pipeline sums `@budget.wasm` across all composed circuits and checks the aggregate against `bundle_class` ceiling:

```
polymail-inbox:     42 KB  (@budget.wasm: 48KB)
polykit-identity:   28 KB  (selective)
polykit-metering:   12 KB  (selective)
polykit-sanitize:   18 KB  (full)
─────────────────────────
Total:             100 KB  (vs. 512 KB app ceiling) ✓
```

**Lock file**: `estream-dev` generates a `circuit.lock` file (similar to `Cargo.lock`) pinning exact versions:

```yaml
# circuit.lock — auto-generated, committed to repo
[[circuit]]
name = "polykit-identity"
version = "0.1.3"
hash = "sha3-256:abc123..."
```

---

### Proposal 7: `wire:` — Lex Topic Contract Declaration

**Problem**: The lex topics a circuit reads/writes are scattered across `emissions:` blocks and implicit in code. There's no single place to see the full topic contract, and no way for the build pipeline to validate topic permissions or detect subscription/emission mismatches.

**Proposed**:
```yaml
wire:
  subscribes:
    - pattern: "{app}.{user_id}.upload"
      type: event
      description: "Incoming file upload requests"
    - pattern: "lex://estream/apps/{app}/eslm/classification"
      type: signal
      description: "ESLM classification suggestions"
  emits:
    - pattern: "lex://estream/apps/{app}/telemetry"
      type: signal
      payload: TelemetryEvent
    - pattern: "{app}.{user_id}.upload.confirm"
      type: event
      payload: UploadConfirmation
  request_reply:
    - pattern: "{app}.{user_id}.download"
      request: DownloadRequest
      reply: DownloadResponse
```

**Codegen impact — typed subscription handlers**:

The ESCIR → Rust codegen generates typed subscription handler stubs and emission helpers:

```rust
// AUTO-GENERATED from wire: section

/// Subscription handler for {app}.{user_id}.upload (event)
/// Developer implements the body.
pub fn on_upload(ctx: &mut Context, event: UploadRequest) -> Result<()> {
    // Developer's logic here
    todo!()
}

/// Typed emission helper for telemetry (generated, not hand-written)
pub fn emit_telemetry(ctx: &mut Context, event: &TelemetryEvent) -> Result<()> {
    let payload = serde_json::to_vec(event)?;
    ctx.emit_sanitized("lex://estream/apps/{app}/telemetry", &payload)
}
```

**Codegen: topic permission manifest**:

The build pipeline extracts the `wire:` contract into the `.escd` manifest, enabling the runtime to validate that a circuit only accesses topics it declared:

```json
{
  "wire_contract": {
    "subscribes": ["polylabs.data.*.upload"],
    "emits": ["lex://estream/apps/polylabs.data/telemetry", "polylabs.data.*.upload.confirm"],
    "request_reply": ["polylabs.data.*.download"]
  }
}
```

Edge nodes use this to enforce topic-level ACLs without application code.

---

### Proposal 8: `@offline` — Offline Capability Declaration

**Problem**: Some circuits must work without wire connectivity (Poly Data offline access, Poly Messenger queued messages, Poly Pass local vault). Today, offline support is ad-hoc — developers manually implement ESLite caching, queue management, and sync-on-reconnect. The circuit definition doesn't express offline capability, so the build pipeline can't include necessary infrastructure.

**Proposed**:
```yaml
annotations:
  "@offline":
    mode: "queue"              # "queue" | "cache" | "full"
    max_queue: 1000            # max queued operations
    sync_on_reconnect: true    # auto-sync when wire reconnects
    conflict_resolution: "lww" # "lww" (last-writer-wins) | "merge" | "manual"
    cache_ttl: "24h"           # how long cached data is valid offline
```

**Mode descriptions**:
- `queue`: operations are queued locally and replayed when online (write-through)
- `cache`: read-only local cache of subscribed stream data
- `full`: bidirectional — reads from cache, writes to queue, auto-sync

**Codegen impact — offline wrapper generation**:

When `@offline` is declared, the codegen wraps the wire layer with offline-aware logic:

```rust
// AUTO-GENERATED from @offline: { mode: "queue" }

pub fn emit_or_queue(ctx: &mut Context, topic: &str, payload: &[u8]) -> Result<()> {
    if ctx.wire.is_connected() {
        ctx.wire.emit(topic, payload)
    } else {
        // Queue to ESLite for replay on reconnect
        ctx.eslite.insert("/polykit/offline_queue", &QueueEntry {
            topic: topic.to_string(),
            payload: payload.to_vec(),
            queued_at: ctx.time(),
            sequence: ctx.offline.next_sequence(),
        })?;
        
        if ctx.offline.queue_len() > 1000 {  // from max_queue
            return Err(PolykitError::OfflineQueueFull);
        }
        Ok(())
    }
}

// AUTO-GENERATED: sync-on-reconnect handler
pub fn on_reconnect(ctx: &mut Context) -> Result<()> {
    let entries = ctx.eslite.query("/polykit/offline_queue")
        .order_by("sequence", Asc)
        .execute()?;
    
    for entry in entries.rows {
        ctx.wire.emit(&entry.topic, &entry.payload)?;
        ctx.eslite.delete("/polykit/offline_queue", &entry.key)?;
    }
    Ok(())
}
```

The codegen also generates the ESLite migration for the offline queue table automatically — developers don't need to define it.

---

### Proposal 9: StreamSight Shorthand on Emissions

**Problem**: Every emission block repeats the same 4 StreamSight annotations. With 5 emissions per circuit and 6 circuits per app, that's 120 StreamSight annotation lines that are mostly identical.

**Current**:
```yaml
emissions:
  - topic: "lex://estream/apps/{app}/telemetry"
    payload: TelemetryEvent
    annotations:
      "@streamsight_filter": "baseline"
      "@streamsight_sensitivity": 2.0
      "@streamsight_warmup": 1000
      "@streamsight_sample_normal": 0.01
```

**Proposed — compound form**:
```yaml
emissions:
  - topic: "lex://estream/apps/{app}/telemetry"
    payload: TelemetryEvent
    streamsight: { filter: baseline, sensitivity: 2.0, warmup: 1000, sample: 0.01 }
```

**Proposed — named preset**:
```yaml
emissions:
  - topic: "lex://estream/apps/{app}/telemetry"
    payload: TelemetryEvent
    streamsight: "standard"         # expands to the 4 default values

  - topic: "lex://estream/apps/{app}/metrics/deviations"
    payload: Deviation
    streamsight: "passthrough"       # filter: none — every event persisted
    
  - topic: "lex://estream/apps/{app}/incidents"
    payload: Incident
    streamsight: "sensitive"         # sensitivity: 1.0, warmup: 100, sample: 0.1
```

**Built-in presets**:
```yaml
# Defined at platform level
streamsight_presets:
  standard:    { filter: baseline, sensitivity: 2.0, warmup: 1000, sample: 0.01 }
  sensitive:   { filter: baseline, sensitivity: 1.0, warmup: 100,  sample: 0.10 }
  passthrough: { filter: none }
  silent:      { filter: discard }   # for internal emissions not observed
```

**Codegen impact**: Pure preprocessing — the compiler expands presets before codegen. Zero impact on Rust/WASM generation. `estream-dev lint` warns on undefined presets.

---

### Codegen Quality Improvements

Beyond annotations, here are codegen-specific ideas:

#### CG-1: Generate `Display` and `From` for circuit-defined types

Currently, ESCIR `types:` sections define structs and enums but the codegen only produces bare Rust structs. Useful trait implementations should be auto-generated:

```yaml
types:
  Classification:
    enum: [Public, Internal, Confidential, Restricted, Sovereign]
```

Should generate:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Classification {
    Public,
    Internal,
    Confidential,
    Restricted,
    Sovereign,
}

impl std::fmt::Display for Classification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => write!(f, "PUBLIC"),
            // ...
        }
    }
}

impl std::str::FromStr for Classification {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "PUBLIC" => Ok(Self::Public),
            // ...
        }
    }
}
```

#### CG-2: Generate TypeScript type bindings alongside Rust

The `estream generate typescript` command (from #555) runs as a separate step. It should be integrated into the main codegen pipeline so Rust and TS types are always in sync:

```yaml
codegen:
  rust: true              # default
  typescript: true        # generate .d.ts alongside .rs
  json_schema: true       # generate JSON schema for IDE validation
```

Output structure:
```
circuits/polykit-metering/
├── circuit.escir.yaml
├── generated/
│   ├── types.rs          # Rust types (for WASM compilation)
│   ├── types.ts          # TypeScript types (for thin TS layer)
│   └── schema.json       # JSON schema (for IDE autocomplete)
```

#### CG-3: Generate test harness stubs from `exports:`

Every circuit export should auto-generate a test stub:

```yaml
exports:
  - fn: record_usage
    params:
      - name: user_id
        type: bytes(16)
      - name: operation
        type: string
      - name: dimensions
        type: DimensionValues
    returns: MeteringRecord
```

Codegen:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_usage_basic() {
        let ctx = TestContext::new();
        let user_id = [0u8; 16];
        let dimensions = DimensionValues::default();
        
        let result = record_usage(&ctx, &user_id, "test_op", &dimensions);
        assert!(result.is_ok());
        
        let record = result.unwrap();
        assert_eq!(record.user_id, user_id);
        assert_eq!(record.operation, "test_op");
    }
    
    // TODO: Add more test cases
}
```

#### CG-4: Generate ESF/ESZ test vectors from type definitions

Each type in the `types:` section should auto-generate ESF (valid) and ESZ (edge case) test vectors:

```rust
// Auto-generated from types: MeteringRecord
pub fn esf_metering_record_vectors() -> Vec<(String, Vec<u8>)> {
    vec![
        ("valid_minimal".into(), MeteringRecord {
            user_id: [0u8; 16],
            operation: "".into(),
            dimensions: DimensionValues::default(),
            timestamp_ms: 0,
        }.to_bytes()),
        ("valid_maximal".into(), MeteringRecord {
            user_id: [0xFF; 16],
            operation: "x".repeat(256),
            dimensions: DimensionValues { executions: u64::MAX, ..Default::default() },
            timestamp_ms: u64::MAX,
        }.to_bytes()),
    ]
}
```

---

## Delta Curate Concept

> Full specification: [`polyquantum/estream-io/specs/architecture/DELTA_CURATE_SPEC.md`](https://github.com/polyquantum/estream-io/blob/main/specs/architecture/DELTA_CURATE_SPEC.md)

**Delta Curate** is a proposed new eStream pattern — a lossless complement to the existing Curate ("Discard Normal") architecture. Where Curate discards statistically normal events (~180x reduction, lossy), Delta Curate **keeps all events but stores only what changed** per record at the field level.

### Core Mechanism

- **Field-level bitmask**: A presence mask indicates which fields changed (zero-delta fields are cancelled — zero bits stored)
- **Variable-width shift encoding**: Each non-zero delta is stored at the minimum bit width needed for its magnitude (a ±1 change costs 2 bits, not 32)
- **Epoch snapshots**: Full records at configurable intervals bound the maximum reconstruction distance
- **Lossless**: Any value at any time is fully reconstructable from epoch + deltas

### Key Innovation: Bitmask Pattern Proof

The bitmask metadata aggregates up the hierarchical Merkle tree, enabling novel proof types:

- **Exclusion proof**: "Field X did NOT change between T1 and T2" — single Merkle path where `or_mask` bit = 0
- **Frequency proof**: "Field X changed K times in period" — aggregate `field_change_counts`
- **Statistical authenticity**: Co-occurrence matrix and shift-level distributions form a **data source fingerprint** committed at epoch boundaries — provable evidence that data has the statistical properties of genuine data from its claimed source

### Composable with Curate

The two patterns chain in the FPGA pipeline: Delta Curate encodes (~20x), then Curate filters normal delta patterns (~20x), yielding **~400x combined reduction** vs raw storage — fully auditable.

### FPGA Resources

- Encoder: ~800 LUTs, 2 BRAMs, 4-5 cycles/record @ 200 MHz (40-50M records/sec)
- Fits alongside baseline-gate on Platypus eFPGA

### Relevance to PolyKit

Delta Curate would benefit Poly Data (document version tracking), Poly Messenger (message delivery telemetry), and any future Poly Labs product handling time series or audit trail data. The encoding/decoding could be exposed via PolyKit crates and the `@delta_curate` ESCIR annotation.

---

## Codegen Quality Observations

*To be filled in once ESCIR → Rust codegen is exercised against PolyKit circuits.*

---

## Testing Observations

*To be filled in once ESF/ESZ test vectors are generated for PolyKit circuits.*
