# PolyKit FastLang Refactor Design

> **Version**: 0.2.0
> **Date**: February 2026
> **Platform**: eStream v0.8.3
> **Epic**: [EPIC_POLYKIT_FASTLANG_REFACTOR](../.github/epics/EPIC_POLYKIT_FASTLANG_REFACTOR.md)
> **Previous**: [ARCHITECTURE.md](ARCHITECTURE.md) (v0.1.0, ESCIR YAML)

---

## Current State

PolyKit v0.1.0 defines 6 shared circuits as `.escir.yaml` files at ESCIR v0.8.1:

| Circuit | YAML File | Exports | Purpose |
|---------|-----------|---------|---------|
| polykit-identity | `circuits/polykit-identity/circuit.escir.yaml` | 6 | SPARK auth + HKDF key derivation |
| polykit-metering | `circuits/polykit-metering/circuit.escir.yaml` | 3 | 8-dimension resource metering |
| polykit-telemetry | `circuits/polykit-telemetry/circuit.escir.yaml` | 4 | StreamSight "Discard Normal" |
| polykit-rate-limiter | `circuits/polykit-rate-limiter/circuit.escir.yaml` | 2 | FIFO rate limiter |
| polykit-sanitize | `circuits/polykit-sanitize/circuit.escir.yaml` | 2 | 3-stage PII/PCI/HIPAA/GDPR |
| polykit-li_effects-classify | `circuits/polykit-li_effects-classify/circuit.escir.yaml` | 3 | ML classification + human-in-loop |

These are declarative YAML: they define types, exports, and emissions but contain **no computation logic**. The hand-written Rust in `crates/` implements the actual behavior (~2,500 lines across 5 crates). Additionally, 4 FastLang `.fl` files exist in `estream-io/crates/estream-fastlang/examples/polylabs/` (media_stream, crdt_sync, blind_relay, classified_fusion) serving as platform-level golden sources.

---

## Target State

**FastLang becomes the single source of truth for all PolyKit circuits.** The `.escir.yaml` files are retired. FastLang generates the Rust implementation, the WASM binaries, and the type bindings -- replacing both the YAML definitions and much of the hand-written Rust crate code.

Key changes:
- 6 YAML circuits become **10 core `.fl` files + 4 app-level circuits** (telemetry eliminated, regional + app circuits added)
- StreamSight telemetry is **inline on every circuit** via annotations (not a separate circuit)
- **Multi-level lex hierarchy**: Global org lex + US regional (CCPA/SOC2) + EU regional (GDPR/EU AI Act)
- **Fan-in/fan-out**: Regional sub-lexes fan in app data; only anonymized metrics fan out to global
- **LI Effects** (Learned Intelligence): `li_classify`, `li_embed`, `li_infer`, `li_train` builtins
- Hand-written Rust shrinks from ~2,500 to ~600 lines (kernel + shim)
- ~1,400 lines of `.fl` generate the rest via codegen
- Build pipeline: `.fl` -> ESCIR -> Rust/WASM/TypeScript codegen -> `.escd`

---

## Design Principle

Same as v0.1.0: **Push everything into Rust/WASM. TypeScript is ONLY a DOM binding layer.** What changes is the *source*: FastLang `.fl` files replace both ESCIR YAML and hand-written Rust for circuit logic. The codegen pipeline produces the same WASM outputs.

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│  Browser / React Native                                          │
│                                                                  │
│  @polykit/react (thin TS — unchanged)                            │
│  ├── PolyProvider          loads polykit.wasm, inits SPARK       │
│  ├── useWasmSubscription   subscribes via WASM wire client       │
│  ├── useWasmEmit           emits via WASM wire client            │
│  └── WidgetShell           layout + RBAC gate                    │
├──────────────────────────────────────────────────────────────────┤
│  polykit.wasm (FastLang → ESCIR → Rust/WASM codegen)            │
│                                                                  │
│  circuits/fl/*.fl          8 circuits + 1 platform composition   │
│  polykit-core              thin kernel (AppContext, helpers)      │
│  polykit-eslite            migrations, queries, sync             │
│  polykit-console           event bus, widget data, demo, RBAC    │
├──────────────────────────────────────────────────────────────────┤
│  eStream Wire Protocol (UDP :5000 / WebTransport :4433)          │
└──────────────────────────────────────────────────────────────────┘
```

---

## Shared Annotation Profile

A v0.8.3 feature: define once, use everywhere. Replaces 5-7 repeated annotations on every circuit.

**`circuits/fl/polykit_profile.fl`**:

```fastlang
amplify enterprise {
    families [lattice, code],
    rounds 2,
}

profile poly_framework_standard {
    lex esn/global/org/polylabs
    precision C
    budget wasm_bytes 64KB, memory_pages 32, instantiation_ms 25
    meters [compute_cycles, memory_bytes, crypto_ops, bandwidth_bytes]
    observe metrics: [circuit_invocations, latency_p99_ms]
    streamsight emit
    rbac [user, operator, admin]
    offline mode queue, sync eventual, max_offline 24h
    wasm_abi [eslite_query, eslite_insert, stream_emit, stream_subscribe]
}

profile poly_framework_sensitive inherits poly_framework_standard {
    constant_time true
    sanitize pii_fields [], retention 7y
    witness threshold(3, 5)
}
```

Every circuit inheriting `poly_framework_standard` gets baseline StreamSight telemetry (`observe metrics`, `streamsight emit`), WASM budget enforcement, metering hooks, RBAC gating, and offline support automatically.

---

## Inline StreamSight Telemetry (No Separate Circuit)

The `polykit-telemetry/circuit.escir.yaml` is **not converted to its own `.fl` file**. StreamSight is designed to be native and inline -- the FastLang compiler automatically weaves telemetry into generated code. This matches the pattern used across all 47 `.fl` files in estream-io.

| Mechanism | Syntax | Applied Where | Effect |
|-----------|--------|---------------|--------|
| Metric declaration | `observe metrics: [...]` | Every circuit annotation block | Declares counters; auto-enables `streamsight_emit` |
| Runtime alert | `monitor "name" { expr }` | Circuits with thresholds | Dual-semantic: SMT proof + StreamSight runtime alert |
| Anomaly detection | `streamsight_anomaly(value)` | In circuit bodies | Checks value against learned baseline |
| Baseline retrieval | `streamsight_baseline("metric")` | In circuit bodies | Retrieves baseline statistics |
| Verification artifact | `esz_emit "path"` | Circuits needing PoVC evidence | Emits .esz file |
| LI pipeline feed | `li_feed full_escir true` | Circuits feeding LI | Streams IR to LI for optimization |

The profile provides baseline telemetry. Individual circuits add domain-specific metrics. The Rust codegen auto-generates a `{CircuitName}StreamSight` struct; WASM codegen adds `streamsight_emit`, `streamsight_anomaly`, `streamsight_baseline` as host imports.

---

## FastLang Circuit Files

### Circuit 1: Identity (`polykit_identity.fl`)

Replaces `circuits/polykit-identity/circuit.escir.yaml` AND most of `crates/polykit-core/src/identity.rs` + `crypto.rs`.

```fastlang
type MasterSeed = bytes(32)
type UserId = bytes(16)

type DerivedKeys = struct {
    user_id: UserId,
    signing_public_key: bytes(2592),
    signing_secret_key: bytes(4896),
    encryption_public_key: bytes(1568),
    encryption_secret_key: bytes(3168),
}

type EncapsulatedKey = struct {
    ciphertext: bytes(1568),
    shared_secret: bytes(32),
}

circuit derive_keys(master_seed: MasterSeed, hkdf_context: bytes(64)) -> DerivedKeys
    profile poly_framework_sensitive
    lex esn/global/org/polylabs/identity
    constant_time true
    observe metrics: [key_derivations, hkdf_ops, signing_key_generations]
    monitor "derivation_latency" { derivation_time_ms < 50 }
    invariant "seed_minimum_entropy" { len(master_seed) >= 32 }
    invariant "key_isolation" { signing_seed != encryption_seed }
    kat_vector "spark_derivation_v1" { ... }
{
    let derived = hkdf_sha3(master_seed, hkdf_context, 64)
    let signing_seed = bit_slice(derived, 0, 256)
    let encryption_seed = bit_slice(derived, 256, 512)
    let (signing_pk, signing_sk) = mldsa_sign(signing_seed)
    let (enc_pk, enc_sk) = mlkem_encaps(encryption_seed)
    let pk_hash = sha3_256(signing_pk)
    let user_id = bit_slice(pk_hash, 0, 128)
    DerivedKeys {
        user_id: user_id,
        signing_public_key: signing_pk,
        signing_secret_key: signing_sk,
        encryption_public_key: enc_pk,
        encryption_secret_key: enc_sk,
    }
}

circuit sign_message(secret_key: bytes(4896), message: bytes) -> bytes(4627)
    profile poly_framework_sensitive
    constant_time true
    observe metrics: [sign_ops]
    invariant "signature_deterministic" { sign(sk, msg) == sign(sk, msg) }
{
    mldsa_sign(secret_key, message)
}

circuit verify_signature(public_key: bytes(2592), message: bytes, signature: bytes(4627)) -> bool
    profile poly_framework_standard
    observe metrics: [verify_ops]
{
    mldsa_verify(message, signature, public_key)
}

circuit encapsulate_key(recipient_pk: bytes(1568)) -> EncapsulatedKey
    profile poly_framework_sensitive
    constant_time true
    observe metrics: [encapsulation_ops]
{
    let kem = mlkem_encaps(recipient_pk)
    EncapsulatedKey {
        ciphertext: kem,
        shared_secret: blake3(kem),
    }
}
```

**Improvements over YAML**: Actual computation logic in the circuit body. `constant_time` for side-channel protection. `kat_vector` for NIST known-answer testing. `invariant` for formal verification of key isolation. Profile inheritance eliminates repeated annotations. Inline `observe`/`monitor` replaces separate telemetry circuit.

---

### Circuit 2: Metering (`polykit_metering.fl`)

Replaces `circuits/polykit-metering/circuit.escir.yaml` AND `crates/polykit-core/src/metering.rs`.

```fastlang
type MeteringDimension = enum {
    Executions, Hashes, Bandwidth, Storage,
    Observables, Proofs, Circuits, MpcSessions,
}

type DimensionValues = struct {
    executions: u64, hashes: u64, bandwidth: u64, storage: u64,
    observables: u64, proofs: u64, circuits: u64, mpc_sessions: u64,
}

type MeteringRecord = struct {
    user_id: bytes(16), operation: string,
    dimensions: DimensionValues, timestamp_ms: u64,
}

stream metering_events: event<MeteringRecord>
    | throttle 1000/s burst 2000

circuit record_usage(user_id: bytes(16), operation: string, dims: DimensionValues) -> MeteringRecord
    profile poly_framework_standard
    meters [compute_cycles, memory_bytes]
    observe metrics: [metering_records, dimension_totals]
    monitor "metering_throughput" { metering_records > 0 }
    wasm_abi [eslite_insert, stream_emit, now]
{
    let ts = now()
    let record = MeteringRecord {
        user_id: user_id, operation: operation,
        dimensions: dims, timestamp_ms: ts,
    }
    emit(metering_events, record)
    record
}

circuit check_limits(current: DimensionValues, limits: DimensionValues) -> [MeteringDimension; 8]
    profile poly_framework_standard
    observe metrics: [limit_checks, limit_violations]
    invariant "no_false_positives" { violation implies current_dim > limit_dim }
{
    parallel for i in 0..8 {
        if current[i] > limits[i] { MeteringDimension::from_index(i) } else { null }
    }
}
```

**Improvements**: `stream` declarations with `emit()` replace `emissions:` in YAML. `parallel for` across 8 dimensions for FPGA-friendly checking. `meters` auto-generates metering hooks. Actual computation replaces stub Rust.

---

### Circuit 3: Rate Limiter (`polykit_rate_limiter.fl`)

Replaces `circuits/polykit-rate-limiter/circuit.escir.yaml`.

```fastlang
type RateWindow = struct {
    count: u64,
    window_start_ms: u64,
    window_size_ms: u64,
}

type RateCheckResult = struct {
    allowed: bool,
    remaining: u64,
    reset_at_ms: u64,
    retry_after_ms: u64,
}

stream rate_state: state<RateWindow>

circuit check_rate(user_id: bytes(16), operation: string, tier: string) -> RateCheckResult
    profile poly_framework_standard
    state_machine rate_fsm {
        initial open
        states [open, throttled, blocked]
        transition open -> throttled when usage > soft_limit
        transition throttled -> blocked when usage > hard_limit
        transition blocked -> open when window_reset
    }
    observe metrics: [rate_checks, throttle_events, blocked_events]
    monitor "throttle_rate" { throttle_events / rate_checks < 0.10 }
    wasm_abi [eslite_query, now]
    invariant "monotonic_consumption" { new_count >= old_count }
{
    let current = load(user_id)
    let ts = now()
    let window = fsm_state(rate_fsm)
    let anomaly = streamsight_anomaly(current)
    let baseline = streamsight_baseline("rate_checks")
    RateCheckResult {
        allowed: window == open,
        remaining: 0,
        reset_at_ms: ts,
        retry_after_ms: 0,
    }
}
```

**Improvements**: `state_machine` annotation (FastLang-only) models rate limiter FSM with verifiable transitions. `streamsight_anomaly()` in body detects abuse patterns. `state` stream type for latest-value semantics.

---

### Circuit 4: Sanitize (`polykit_sanitize.fl`)

Replaces `circuits/polykit-sanitize/circuit.escir.yaml` AND `crates/polykit-sanitize/`.

```fastlang
type DataType = enum {
    Ssn, CreditCard, PersonalName, Email, PhoneNumber,
    DateOfBirth, Address, MedicalRecord, FinancialAccount, BiometricData,
}

type Regulation = enum { Hipaa, PciDss, Gdpr, Soc2, Ccpa }
type Stage = enum { PiiDetect, ValueTransform, AuditRecord }

type Detection = struct {
    field_path: string,
    data_type: DataType,
    regulation: [Regulation; 5],
    confidence: u64,
}

type AuditEntry = struct {
    timestamp_ms: u64,
    stage: Stage,
    field_path: string,
    original_type: string,
    placeholder: string,
    regulations: [string; 5],
    witness_hash: bytes(32),
}

type SanitizationResult = struct {
    sanitized_data: bytes,
    audit_entries: [AuditEntry; 32],
}

circuit sanitize(input: bytes) -> SanitizationResult
    profile poly_framework_sensitive
    sanitize pii_fields [input], retention 7y, compliance [hipaa, pci_dss, gdpr, soc2]
    observe metrics: [sanitization_runs, pii_detections, transform_ops]
    monitor "pii_leak_rate" { pii_in_output == 0 }
    witness full
    invariant "no_pii_in_output" { pii_count(sanitized_data) == 0 }
    invariant "audit_complete" { len(audit_entries) >= detections }
    esz_emit "verify/polykit_sanitize.esz"
    li_feed full_escir true, optimized_ir true
{
    let detections = li_classify(input, "pii_scan")
    let transformed = transform_detected(input, detections)
    let audit = generate_audit(detections)
    SanitizationResult { sanitized_data: transformed, audit_entries: audit }
}

circuit detect_only(input: bytes) -> [Detection; 32]
    profile poly_framework_standard
    observe metrics: [detection_only_runs]
{
    li_classify(input, "pii_scan")
}
```

**Improvements**: `@sanitize` auto-generates exhaustiveness checks (bypassing sanitization produces compile error `ESCIR-S001`). `li_classify` for ML-based PII detection (was hand-coded regex). `witness full` for PoVC-witnessed audit trail. `invariant` proves no PII leaks. Full `esz_emit` + `li_feed` verification pipeline.

---

### Circuit 5: LI Effects Classify (`polykit_li_effects_classify.fl`)

Replaces `circuits/polykit-li_effects-classify/circuit.escir.yaml`.

```fastlang
type ClassificationSuggestion = struct {
    tag: string,
    confidence: u64,
    alternatives: [Alternative; 5],
}

type Alternative = struct {
    tag: string,
    confidence: u64,
}

type HumanFeedback = struct {
    sample_hash: bytes(32),
    rating: u8,
    correction: string,
    reviewer_hash: bytes(32),
    timestamp_ms: u64,
}

type ConfidenceThresholds = struct {
    auto_accept: u64,
    review_required: u64,
    auto_reject: u64,
}

stream classification_events: event<ClassificationSuggestion>
    classify tag {
        field_governance {
            tag: public,
            confidence: public,
            alternatives: encrypted(audience: internal),
        }
    }

circuit classify_content(content: bytes, filename: string, metadata: bytes(256)) -> ClassificationSuggestion
    profile poly_framework_standard
    observe metrics: [classifications, confidence_distribution, auto_accept_rate]
    monitor "confidence_drift" { avg_confidence > 0.60 }
    feedback classification_events -> li_train
{
    let embedding = li_embed(content, filename)
    let suggestion = li_classify(content, metadata)
    let confidence = li_infer(embedding, suggestion)
    ClassificationSuggestion { tag: suggestion, confidence: confidence, alternatives: [] }
}

circuit submit_feedback(feedback: HumanFeedback) -> bool
    profile poly_framework_standard
    wasm_abi [eslite_insert, stream_emit]
    observe metrics: [feedback_submitted, correction_rate]
{
    let stored = store(feedback.sample_hash, feedback)
    emit(classification_events, feedback)
    true
}

circuit get_thresholds() -> ConfidenceThresholds
    profile poly_framework_standard
    observe metrics: [threshold_queries]
    wasm_abi [eslite_query]
{
    let thresholds = load("confidence_thresholds")
    thresholds
}
```

**Improvements**: `feedback` stream operator loops classification results back into LI training. `li_embed`, `li_classify`, `li_infer` builtins replace hand-rolled ML stubs. `classify` + `field_governance` for field-level access control on suggestions. `monitor` tracks confidence drift.

---

### NEW Circuit 6: Delta Curate (`polykit_delta_curate.fl`)

Leverages the Delta Curate concept from [ESTREAM_FEEDBACK.md](ESTREAM_FEEDBACK.md). Not expressible in YAML.

```fastlang
type DeltaRecord = struct {
    bitmask: u64,
    deltas: bytes(512),
    epoch: u64,
    is_snapshot: bool,
}

type ReconstructedRecord = struct {
    data: bytes(4096),
    epoch: u64,
    delta_count: u64,
}

circuit delta_encode(current: bytes(4096), previous: bytes(4096), epoch: u64) -> DeltaRecord
    profile poly_framework_standard
    delta_curate {
        epoch_interval 100,
        bitmask_width 64,
        proof_mode exclusion,
    }
    observe metrics: [delta_ratio, epoch_snapshots, bitmask_density]
    monitor "compression_ratio" { delta_ratio > 5 }
    invariant "lossless" { decode(encode(record)) == record }
    esz_emit "verify/polykit_delta_curate.esz"
{
    let changed = bit_slice(current, 0, 64) ^ bit_slice(previous, 0, 64)
    let bitmask = changed
    let deltas = extract_deltas(current, previous, bitmask)
    let is_snapshot = epoch % 100 == 0
    DeltaRecord {
        bitmask: bitmask,
        deltas: deltas,
        epoch: epoch,
        is_snapshot: is_snapshot,
    }
}

circuit delta_decode(snapshot: bytes(4096), deltas: [DeltaRecord; 100]) -> ReconstructedRecord
    profile poly_framework_standard
    observe metrics: [reconstruction_ops]
    invariant "reconstruction_deterministic" { decode(s, d) == decode(s, d) }
{
    let data = apply_deltas(snapshot, deltas)
    let count = len(deltas)
    ReconstructedRecord { data: data, epoch: 0, delta_count: count }
}
```

**Purpose**: ~20x lossless compression for document version tracking (Poly Data), message delivery telemetry (Poly Messenger), and audit-trail data. Bitmask Pattern Proof enables exclusion proofs ("field X did NOT change between T1-T2"). Composable with Curate for ~400x combined reduction.

---

### NEW Circuit 7: Field Governance (`polykit_governance.fl`)

Field-level visibility control with filtered fan-out. Not expressible in YAML.

```fastlang
audience public { fields [summary, status, timestamp] }
audience internal inherits public { fields [details, assignee, priority] }
audience privileged inherits internal { fields [pii_data, raw_content], hide [internal_notes] }

type GovernedRecord = struct {
    summary: string,
    status: string,
    timestamp: u64,
    details: bytes(1024),
    assignee: bytes(32),
    priority: u8,
    pii_data: bytes(512),
    raw_content: bytes(4096),
    internal_notes: bytes(1024),
}

circuit governed_emit(record: GovernedRecord, classification: string) -> bytes(32)
    profile poly_framework_standard
    lex esn/global/org/polylabs {
        governance hierarchical
        audit_trail true
        sub_lex app fan_out share [summary, status] redact [pii_data, raw_content]
    }
    field_governance {
        summary: public,
        status: public,
        timestamp: public,
        details: encrypted(audience: internal),
        assignee: encrypted(audience: internal),
        priority: encrypted(audience: internal),
        pii_data: hidden(audience: privileged),
        raw_content: redacted,
        internal_notes: redacted,
    }
    observe metrics: [governed_emissions, field_redactions, audience_checks]
    monitor "redaction_coverage" { field_redactions > 0 }
    invariant "no_pii_at_public" { pii_visible_to_public == false }
    property safety "classification_monotonic" { output_level >= input_level }
{
    let record_hash = sha3_256(record)
    let classified = li_classify(record, classification)
    emit(record, classified)
    record_hash
}
```

**Purpose**: Shared field-level governance primitive for all Poly Labs apps. 2 bits per field per audience, FPGA wire-speed enforcement. Apps compose this into their domain circuits.

---

### Platform Composition (`polykit_platform.fl`)

Ties all circuits together:

```fastlang
platform polykit_framework v1 {
    import polykit_identity from "./polykit_identity.fl"
    import polykit_metering from "./polykit_metering.fl"
    import polykit_rate_limiter from "./polykit_rate_limiter.fl"
    import polykit_sanitize from "./polykit_sanitize.fl"
    import polykit_li_effects_classify from "./polykit_li_effects_classify.fl"
    import polykit_delta_curate from "./polykit_delta_curate.fl"
    import polykit_governance from "./polykit_governance.fl"

    group core [polykit_identity, polykit_metering, polykit_rate_limiter]
    group compliance [polykit_sanitize, polykit_governance]
    group intelligence [polykit_li_effects_classify, polykit_delta_curate]

    connect polykit_identity -> polykit_metering
    connect polykit_metering -> polykit_rate_limiter
    connect polykit_sanitize -> polykit_governance

    bind polykit_li_effects_classify -> li_engine

    streamsight true
    esz_emit "verify/polykit_platform.esz"
    li_feed full_escir true, optimized_ir true

    target wasm_client
    target alpha_devnet
}
```

---

## Updated Build Pipeline

```
circuits/fl/*.fl
  → estream codegen compile (parse, type-check, lower to ESCIR)
  → ESCIR optimization passes (O2)
  → Rust codegen (replaces most hand-written crate code)
  → cargo build --target wasm32-unknown-unknown
  → wasm-opt -Oz
  → ABI validation + size budget check
  → ML-DSA-87 signing
  → .escd packaging
  → TypeScript .d.ts generation (new in v0.8.3)
```

Single command:

```bash
estream-dev build-wasm-client --from-fl circuits/fl/ --sign key.pem --enforce-budget
```

---

## What Stays Hand-Written

| Component | Purpose | Estimated Size |
|-----------|---------|---------------|
| `polykit-core` | Thin kernel: `AppContext`, `format_user_topic()`, `format_global_topic()`, error types | ~200 lines |
| `polykit-wasm` | wasm-bindgen shim routing JS calls to codegen'd exports | ~150 lines |
| `polykit-eslite` | ESLite runtime (migrations, queries, sync) | ~250 lines (unchanged) |
| `polykit-console` | Widget runtime (event bus, data pipeline, RBAC) | ~300 lines (unchanged) |
| `packages/react` | `PolyProvider`, hooks, `WidgetShell` | Unchanged |

Total: ~600 lines hand-written Rust (down from ~2,500) + ~800 lines `.fl` circuits.

---

## File Structure After Refactor

```
polykit/
  .github/
    epics/
      EPIC_POLYKIT_FASTLANG_REFACTOR.md
  circuits/
    fl/
      polykit_profile.fl          # Shared annotation profiles
      polykit_identity.fl         # SPARK + HKDF + ML-DSA/ML-KEM
      polykit_metering.fl         # 8-dimension metering
      polykit_rate_limiter.fl     # Rate limiter with FSM
      polykit_sanitize.fl         # 3-stage compliance
      polykit_li_effects.fl        # Learned Intelligence classification + feedback
      polykit_delta_curate.fl     # Delta curation (NEW)
      polykit_governance.fl       # Field governance + regional fan-in (NEW)
      polykit_regional_us.fl      # US sovereignty, CCPA, SOC2 (NEW)
      polykit_regional_eu.fl      # EU sovereignty, GDPR, EU AI Act (NEW)
      polykit_platform.fl         # Platform composition (core + regional + apps)
      apps/
        polykit_media_stream.fl   # PQ-encrypted voice/video SFU
        polykit_crdt_sync.fl      # Offline-capable CRDT merge
        polykit_blind_relay.fl    # Privacy-preserving message relay
        polykit_classified_fusion.fl  # SCI omniscient fusion
    legacy/                       # Archived .escir.yaml (reference only)
      polykit-identity/
      polykit-metering/
      polykit-telemetry/          # Eliminated as separate circuit
      polykit-rate-limiter/
      polykit-sanitize/
      polykit-eslm-classify/        # Legacy name; now polykit_li_effects.fl
  crates/
    polykit-core/                 # Slimmed: kernel helpers only
    polykit-wasm/                 # Slimmed: thin shim over codegen
    polykit-eslite/               # Mostly unchanged (ESLite runtime)
    polykit-console/              # Mostly unchanged (widget runtime)
  packages/react/                 # Unchanged
  docs/
    ARCHITECTURE.md               # Updated for FastLang pipeline
    ESTREAM_GETTING_STARTED.md    # Updated build commands + circuit catalog
    ESTREAM_FEEDBACK.md           # Unchanged (running feedback)
    FASTLANG_REFACTOR_PLAN.md     # This document
```

---

## How Apps Use PolyKit (Updated)

Apps depend on PolyKit `.fl` circuits via composition and extend with domain-specific circuits:

```fastlang
// App circuit composes PolyKit shared circuits
circuit polymail_inbox(user_id: bytes(16), message: bytes) -> bytes(32)
    profile poly_framework_standard
    composes: [polykit_identity, polykit_metering, polykit_sanitize]
    lex esn/global/org/polylabs/polymail
    observe metrics: [inbox_messages, message_size_avg]
{
    let sanitized = sanitize(message)
    let metered = record_usage(user_id, "inbox_receive", dims)
    sha3_256(sanitized)
}
```

```tsx
// App TS — minimal DOM binding (unchanged)
import { PolyProvider } from '@polykit/react';

export default function App() {
  return (
    <PolyProvider wasm="/pkg/polymail.wasm" hkdfContext="poly-mail-v1">
      <WidgetGrid />
    </PolyProvider>
  );
}
```

---

## Related Documents

| Document | Purpose |
|----------|---------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | Current v0.1.0 architecture (ESCIR YAML) |
| [ESTREAM_FEEDBACK.md](ESTREAM_FEEDBACK.md) | Running feedback on eStream platform |
| [ESTREAM_GETTING_STARTED.md](ESTREAM_GETTING_STARTED.md) | Getting started guide (to be updated) |
| [FastLang Spec](https://github.com/polyquantum/estream-io/blob/main/specs/protocol/FASTLANG_SPEC.md) | Canonical FastLang language specification |
| [FastLang Quickstart](https://github.com/polyquantum/estream-io/blob/main/docs/guides/FASTLANG_QUICKSTART.md) | Zero to compiled circuit in 15 minutes |
| [ESCIR WASM Client Spec](https://github.com/polyquantum/estream-io/blob/main/specs/architecture/ESCIR_WASM_CLIENT_SPEC.md) | Full WASM client specification |
| [Native Transition Epic](https://github.com/polyquantum/estream-io/blob/main/.github/epics/EPIC_FASTLANG_NATIVE_TRANSITION.md) | Broader .fl-first migration strategy |
