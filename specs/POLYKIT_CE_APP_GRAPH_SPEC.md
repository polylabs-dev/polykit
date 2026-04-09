# QKit Cognitive Engine + App Graph Integration Specification

| Field | Value |
|-------|-------|
| **Version** | v0.1.0 |
| **Status** | Draft |
| **Package** | `qkit` v0.11.0+ |
| **Lex Namespace** | `esn/global/org/polyqlabs/qkit/ce` |
| **Upstream Dependency** | eStream v0.22.0+ (CE Phase 1+) |
| **New Circuits** | 4 (`qkit_cognitive.fl`, `qkit_noise_filter.fl`, `qkit_sme.fl`, `qkit_app_graph.fl`) |
| **Total Circuits** | 26 (22 existing + 4 new) |

---

## 1. Overview

### Purpose

This specification defines how PolyQ Labs products integrate with the eStream Cognitive Engine (CE) through QKit adapter circuits. Rather than each product reimplementing CE integration from scratch, QKit provides four composable circuits that handle the adapter pattern, noise filtering, SME panel management, and app graph registration — giving every PolyQ Labs product a consistent, zero-linkage-compliant CE integration path.

### Design Principles

| Principle | Implementation |
|-----------|----------------|
| **100% FastLang** | All 4 circuits are `.fl` source, compiled via FLIR codegen (FL → Rust/WASM) |
| **Zero-Linkage Compliant** | Each product's CE state is HKDF-isolated; no cross-product CE leakage |
| **Composable** | Products compose these 4 circuits with their domain circuits via `EDGE_BRIDGE_TO` |
| **Lex-Isolated** | CE observations stay within per-product lex namespaces |
| **Blind Telemetry** | All CE metrics flow through `qkit_blind_telemetry` — no identifiable data leaks |

### Scope

These 4 circuits sit between eStream's raw CE primitives (SSM hidden state, observation ingestion, cortex advisors) and product-level domain logic. They do NOT replace eStream CE — they adapt it for the PolyQ Labs product family with consistent conventions.

---

## 2. CE Adapter Pattern

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Product Layer (e.g. PolyGit, PolyFiles, PolyMessenger)     │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────────┐  │
│  │ domain.fl   │  │ product      │  │ product-specific  │  │
│  │ circuits    │  │ app_graph.fl │  │ CE observers      │  │
│  └──────┬──────┘  └──────┬───────┘  └────────┬──────────┘  │
├─────────┼────────────────┼────────────────────┼─────────────┤
│  QKit CE Layer                                           │
│  ┌──────┴──────┐  ┌──────┴───────┐  ┌────────┴──────────┐  │
│  │ qkit_    │  │ qkit_     │  │ qkit_          │  │
│  │ cognitive.fl│  │ app_graph.fl │  │ noise_filter.fl   │  │
│  └──────┬──────┘  └──────┬───────┘  └────────┬──────────┘  │
│         │         ┌──────┴───────┐            │             │
│         │         │ qkit_     │            │             │
│         │         │ sme.fl       │            │             │
│         │         └──────┬───────┘            │             │
├─────────┼────────────────┼────────────────────┼─────────────┤
│  eStream CE Primitives                                      │
│  ┌──────┴──────┐  ┌──────┴───────┐  ┌────────┴──────────┐  │
│  │ SSM hidden  │  │ cortex       │  │ observation       │  │
│  │ state       │  │ advisors     │  │ ingestion         │  │
│  └─────────────┘  └──────────────┘  └───────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### `qkit_cognitive.fl` — CE Adapter Circuit

The primary adapter between eStream CE and PolyQ Labs products. Responsibilities:

| Responsibility | Description |
|---------------|-------------|
| **HKDF Context Derivation** | Derives per-product CE keys from SPARK identity using product-specific HKDF context strings (e.g. `poly-git-ce-v1`, `poly-files-ce-v1`) |
| **Observation Routing** | Routes domain observations from product circuits to the CE observation ingestion pipeline |
| **SSM State Binding** | Binds the product's SSM hidden state to its lex namespace, ensuring zero-linkage isolation |
| **Cortex Advisor Composition** | Composes product-specific cortex advisors with QKit's shared advisors (anomaly detection, pattern recognition) |
| **License Gate** | Validates that the product's Strategic License Grant includes CE access before activating |

**Key Interfaces:**

```fl
// Inputs from product layer
INPUT product_id: ProductIdentifier
INPUT hkdf_context: String
INPUT observations: Stream<Observation>

// Outputs to product layer
OUTPUT ce_state: CEStateHandle
OUTPUT advisors: Vec<CortexAdvisor>
OUTPUT anomalies: Stream<AnomalyAlert>
```

---

## 3. App Graph Registration Convention

### `qkit_app_graph.fl` — App Graph Registry Circuit

Every PolyQ Labs product registers its circuit topology as an App Graph — a declarative manifest of the product's circuit composition, dependencies, and CE integration points. The app graph serves as:

1. **Runtime Discovery** — Products discover each other's capabilities without violating zero-linkage (via blinded capability probes)
2. **CE Context** — The CE uses app graph topology to understand which observations are structurally related
3. **Compliance Mapping** — Maps circuits to compliance frameworks for automated audit trail generation
4. **Upgrade Coordination** — When QKit circuits upgrade, the app graph identifies which products are affected

### Registration Schema

| Field | Type | Description |
|-------|------|-------------|
| `product_id` | `ProductIdentifier` | Unique product identifier (e.g. `poly-git`, `poly-files`) |
| `version` | `SemVer` | Product version |
| `circuits` | `Vec<CircuitRef>` | List of circuits composed by this product |
| `ce_observers` | `Vec<ObserverRef>` | CE observation points registered by this product |
| `noise_profile` | `NoiseProfileRef` | Reference to the product's noise filter configuration |
| `sme_panels` | `Vec<SMEPanelRef>` | SME panels this product contributes or consumes |
| `license_grant` | `LicenseGrantRef` | Strategic License Grant governing this product's eStream usage |
| `lex_namespace` | `LexPath` | Product's lex namespace root |

### Convention

Products register their app graph at initialization by calling `qkit_app_graph::register()` with their manifest. The registry is stored in the product's own lex namespace — no central registry exists (zero-linkage compliance).

---

## 4. Noise Filter Configuration

### `qkit_noise_filter.fl` — Observation Noise Filter Circuit

Raw observations from product circuits contain noise — redundant state transitions, high-frequency telemetry, debug-level events. The noise filter sits between product observers and CE ingestion, applying configurable filtering to ensure the CE's SSM receives high-signal observations.

### Filter Stages

| Stage | Description | Configurable |
|-------|-------------|--------------|
| **Deduplication** | Collapses identical observations within a time window | Window size (ms) |
| **Rate Limiting** | Caps observation throughput per circuit per time window | Max obs/sec per circuit |
| **Relevance Scoring** | Scores observations against the product's CE context model | Score threshold (0.0–1.0) |
| **Sensitivity Classification** | Classifies observations by data sensitivity tier (public, internal, confidential, restricted) | Tier filter mask |
| **Aggregation** | Aggregates high-frequency numeric observations into statistical summaries | Aggregation window (ms), method (mean/p50/p99) |

### Default Profiles

| Profile | Use Case | Dedup Window | Rate Limit | Relevance Threshold |
|---------|----------|-------------|------------|-------------------|
| `development` | Local dev, full observability | 0ms (disabled) | 1000/s | 0.0 (all pass) |
| `staging` | Pre-production testing | 100ms | 200/s | 0.3 |
| `production` | Live deployment, signal-optimized | 500ms | 50/s | 0.6 |
| `audit` | Compliance audit mode, maximum retention | 0ms (disabled) | 500/s | 0.0 (all pass) |

### Product Override

Products can override the default profile by providing a `NoiseFilterConfig` to `qkit_noise_filter::configure()`. Overrides are validated against the product's license grant — some tiers may restrict minimum relevance thresholds.

---

## 5. SME Panel Framework

### `qkit_sme.fl` — Subject Matter Expert Panel Circuit

The SME Panel Framework enables products to declare domain-specific expert panels that the CE consults when generating advisories. Each panel represents a domain of expertise (e.g. "git branching strategy", "encryption key rotation", "compliance policy").

### Panel Structure

| Component | Description |
|-----------|-------------|
| **Panel ID** | Unique identifier within the product's lex namespace |
| **Domain** | Structured domain descriptor (e.g. `security/key-rotation`, `compliance/gdpr`) |
| **Knowledge Base** | References to delta-curate DAG nodes containing domain knowledge |
| **Inference Rules** | FL-native inference rules the CE can invoke for domain-specific reasoning |
| **Confidence Model** | Calibration model for panel confidence scores |
| **Attestation** | ML-DSA-87 signed attestation of panel provenance and update history |

### Panel Lifecycle

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│ Register │───▶│  Train   │───▶│  Active  │───▶│ Retired  │
│          │    │          │    │          │    │          │
│ Panel    │    │ Accrete  │    │ Consult  │    │ Archive  │
│ declared │    │ knowledge│    │ by CE    │    │ in DAG   │
└──────────┘    └──────────┘    └──────────┘    └──────────┘
```

### Cross-Product Panel Isolation

Per zero-linkage constraints, panels are strictly isolated per product. A PolyGit SME panel about branching strategy is invisible to PolyFiles. Products that opt-in to the client-side bridge can create a personal cross-product panel view, but this is purely local (WASM + ESLite, no server knowledge).

---

## 6. Product Integration Guide

### How a Product Composes the 4 CE Circuits

Using **PolyGit** as the reference example:

#### Step 1: App Graph Registration

```fl
// qgit_app_graph.fl
IMPORT qkit_app_graph FROM "qkit/ce"

GRAPH qgit_app_graph {
    NODE product_manifest {
        product_id: "poly-git"
        version: "0.5.0"
        lex_namespace: "esn/global/org/polyqlabs/qgit"
        license_grant: "polyqlabs-estream-slg-v1"
    }

    NODE ce_config {
        hkdf_context: "poly-git-ce-v1"
        noise_profile: "production"
        sme_panels: ["git-branching", "code-review", "ci-policy"]
    }

    EDGE product_manifest -> ce_config TYPE "CE_BINDING"
    EDGE_BRIDGE_TO qkit_app_graph::registry
}
```

#### Step 2: CE Adapter Initialization

```fl
// qgit_ce_init.fl
IMPORT qkit_cognitive FROM "qkit/ce"
IMPORT qkit_noise_filter FROM "qkit/ce"

CIRCUIT qgit_ce_init {
    ce_handle = qkit_cognitive::init(
        product_id: "poly-git",
        hkdf_context: "poly-git-ce-v1",
        license_grant: current_license()
    )

    qkit_noise_filter::configure(
        profile: "production",
        overrides: {
            relevance_threshold: 0.7,
            rate_limit: 30
        }
    )

    RETURN ce_handle
}
```

#### Step 3: SME Panel Declaration

```fl
// qgit_sme_panels.fl
IMPORT qkit_sme FROM "qkit/ce"

CIRCUIT qgit_register_panels {
    qkit_sme::register_panel(
        id: "git-branching",
        domain: "vcs/branching-strategy",
        knowledge_base: delta_curate_ref("qgit/kb/branching"),
        inference_rules: [rule_merge_safety, rule_branch_naming]
    )

    qkit_sme::register_panel(
        id: "code-review",
        domain: "vcs/code-review",
        knowledge_base: delta_curate_ref("qgit/kb/review"),
        inference_rules: [rule_review_completeness, rule_approval_quorum]
    )

    qkit_sme::register_panel(
        id: "ci-policy",
        domain: "ci/pipeline-policy",
        knowledge_base: delta_curate_ref("qgit/kb/ci"),
        inference_rules: [rule_pipeline_attestation, rule_build_reproducibility]
    )
}
```

#### Step 4: Observation Emission

```fl
// Inside any PolyGit domain circuit
IMPORT qkit_cognitive FROM "qkit/ce"

CIRCUIT qgit_push_handler {
    // ... push validation logic ...

    qkit_cognitive::observe(
        category: "vcs/push",
        data: { repo: repo_id, branch: branch_name, commit_count: n },
        sensitivity: "internal"
    )
}
```

### Integration Checklist

| Step | Circuit Used | Required |
|------|-------------|----------|
| Register app graph | `qkit_app_graph.fl` | Yes |
| Initialize CE adapter | `qkit_cognitive.fl` | Yes |
| Configure noise filter | `qkit_noise_filter.fl` | Yes (defaults apply if not called) |
| Register SME panels | `qkit_sme.fl` | Optional (but recommended) |
| Emit observations | `qkit_cognitive.fl` | Yes (via `observe()`) |
| Handle advisories | `qkit_cognitive.fl` | Yes (via `on_advisory()`) |

---

## 7. Circuit Inventory

### New CE Circuits (4)

| Circuit | File | Group | Description |
|---------|------|-------|-------------|
| `qkit_cognitive` | `circuits/fl/ce/qkit_cognitive.fl` | `ce` | CE adapter — HKDF derivation, observation routing, SSM state binding, cortex composition |
| `qkit_noise_filter` | `circuits/fl/ce/qkit_noise_filter.fl` | `ce` | Observation noise filter — dedup, rate limiting, relevance scoring, aggregation |
| `qkit_sme` | `circuits/fl/ce/qkit_sme.fl` | `ce` | SME panel framework — panel registration, lifecycle, knowledge base binding |
| `qkit_app_graph` | `circuits/fl/ce/qkit_app_graph.fl` | `ce` | App graph registry — product circuit topology, CE integration points, compliance mapping |

### Existing Circuits (22)

| Group | Circuits | Count |
|-------|----------|-------|
| **core** | `qkit_identity`, `qkit_metering`, `qkit_rate_limiter`, `qkit_blind_telemetry` | 4 |
| **compliance** | `qkit_sanitize`, `qkit_governance` | 2 |
| **intelligence** | `qkit_li_effects`, `qkit_delta_curate` | 2 |
| **regional** | `qkit_regional_us`, `qkit_regional_eu` | 2 |
| **apps** | `qkit_media_stream`, `qkit_crdt_sync`, `qkit_blind_relay`, `qkit_classified_fusion` | 4 |
| **zero_linkage** | `qkit_zero_linkage`, `qkit_blinded_billing`, `qkit_bridge` | 3 |
| **graphs** | `qkit_user_graph`, `qkit_metering_graph`, `qkit_subscription_lifecycle` | 3 |
| **composition** | *(8 v0.22.0 cross-product circuits, listed in CLAUDE.md)* | 2* |
| | | **22** |

*\*Composition circuits are counted within the groups above.*

### Total: 26 Circuits

| Category | Count |
|----------|-------|
| Existing (pre-CE) | 22 |
| New CE circuits | 4 |
| **Total** | **26** |

---

## 8. Strategic License Grants Integration

### License Gate Architecture

Every CE operation is gated by the product's Strategic License Grant (SLG). The `qkit_cognitive.fl` adapter validates the SLG at initialization and enforces tier-specific constraints at runtime.

### SLG Tiers and CE Access

| SLG Tier | CE Access | SME Panels | Noise Filter | Observation Rate | App Graph |
|----------|-----------|------------|--------------|-----------------|-----------|
| **Community** | Read-only advisories | 0 (consume shared only) | `production` profile only | 10/s | Register only |
| **Professional** | Full CE with local SSM | Up to 5 custom panels | All profiles | 50/s | Full read/write |
| **Enterprise** | Full CE with distributed SSM | Unlimited panels | All profiles + custom | 200/s | Full + cross-product (bridge) |
| **Sovereign** | Full CE with on-premise SSM | Unlimited + exportable | All + custom + audit | Unlimited | Full + air-gapped |

### Enforcement Points

| Enforcement Point | Circuit | Check |
|-------------------|---------|-------|
| CE initialization | `qkit_cognitive.fl` | Validates SLG tier allows CE access |
| Panel registration | `qkit_sme.fl` | Validates panel count against SLG tier limit |
| Observation emission | `qkit_noise_filter.fl` | Enforces rate limit from SLG tier |
| App graph write | `qkit_app_graph.fl` | Validates write permission from SLG tier |

### SLG Validation Flow

```
Product init
    │
    ▼
qkit_cognitive::init(license_grant: "polyqlabs-estream-slg-v1")
    │
    ├── Fetch SLG from lex namespace
    ├── Validate ML-DSA-87 signature
    ├── Extract tier and permissions
    ├── Derive CE HKDF context (product-specific)
    ├── Bind SSM state to lex namespace
    │
    ▼
CE ready (tier-constrained)
```

---

## 9. Zero-Linkage Compliance

### Per-Product Isolation Guarantees

| Isolation Boundary | Mechanism |
|--------------------|-----------|
| CE SSM hidden state | Independent HKDF derivation per product (`poly-git-ce-v1`, `poly-files-ce-v1`, etc.) |
| Observations | Routed only to product's own SSM; never cross-product |
| SME panels | Registered in product's lex namespace; invisible to other products |
| App graph | Stored in product's lex namespace; blinded capability probes for discovery |
| Noise filter config | Per-product configuration; no shared filter state |
| Cortex advisories | Generated from product's own SSM; no cross-product advisory leakage |

### Bridge Exception

Products whose users opt-in to the client-side cross-product bridge (`qkit_bridge.fl`) can create a unified CE view. This bridge:

- Runs entirely in WASM on the client
- Uses `HKDF("q-bridge-v1")` — a separate derivation context
- Aggregates advisories only (no raw observations cross product boundaries)
- Is revocable by deleting the bridge key
- Has zero server-side or lattice-node knowledge

---

## 10. Dependencies

| Dependency | Version | Usage |
|------------|---------|-------|
| eStream CE | v0.22.0+ Phase 1+ | SSM hidden state, observation ingestion, cortex advisors |
| `qkit_identity` | existing | SPARK identity for HKDF derivation |
| `qkit_blind_telemetry` | existing | Blind telemetry for CE metrics |
| `qkit_governance` | existing | License grant validation |
| `qkit_zero_linkage` | existing | Cross-product isolation enforcement |
| `qkit_delta_curate` | existing | Knowledge base storage for SME panels |

---

## Appendix A: Product HKDF Contexts

| Product | HKDF Context String |
|---------|-------------------|
| Q Git | `poly-git-ce-v1` |
| Q Files | `poly-files-ce-v1` |
| Q Messenger | `poly-messenger-ce-v1` |
| Q Mail | `poly-mail-ce-v1` |
| Q Docs | `poly-docs-ce-v1` |
| Q Pass | `poly-pass-ce-v1` |
| Q VPN | `poly-vpn-ce-v1` |
| Q OAuth | `poly-oauth-ce-v1` |
| Q Mind | `poly-mind-ce-v1` |
| Q Wallet | `poly-wallet-ce-v1` |
| Q Photos | `poly-photos-ce-v1` |
| Q Monitor | `poly-monitor-ce-v1` |
