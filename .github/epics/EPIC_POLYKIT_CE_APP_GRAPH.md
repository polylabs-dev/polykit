# QKit Cognitive Engine + App Graph Framework

> **Status**: In Progress
> **Priority**: P0
> **Estimated Effort**: ~3-4 weeks (4 phases)
> **Target**: QKit v0.12.0 on eStream v0.22.0+
> **Depends On**: [eStream CE Implementation (Phase 1+)](https://github.com/polyquantum/estream/blob/main/.github/epics/EPIC_COGNITIVE_ENGINE.md)

---

## Overview

Integrate the eStream Cognitive Engine (CE) into QKit as a shared framework layer, giving every PolyQ Labs product consistent, zero-linkage-compliant access to CE capabilities — SSM hidden state, observation routing, cortex advisors, and accretive learning. This epic introduces 4 new FastLang circuits (`qkit_cognitive.fl`, `qkit_noise_filter.fl`, `qkit_sme.fl`, `qkit_app_graph.fl`) that adapt eStream CE primitives into composable building blocks, plus an App Graph registration convention that enables products to declare their circuit topology and CE integration points. The framework brings QKit from 22 to 26 circuits and establishes the pattern all 38 PolyQ Labs products will follow for CE adoption.

---

## Phase 1: Design & Scaffolding

- [x] Design CE adapter pattern and zero-linkage constraints
- [x] Create `circuits/fl/ce/` directory for CE circuit group
- [x] Define App Graph registration schema and convention
- [ ] Review CE adapter pattern with eStream core team

### Exit Criteria

- CE adapter architecture approved
- `circuits/fl/ce/` directory exists with group structure

---

## Phase 2: CE Adapter Circuits

Create the 4 new FastLang circuits that compose eStream CE primitives into the QKit shared framework.

- [x] `qkit_cognitive.fl` — CE adapter circuit (HKDF derivation, observation routing, SSM state binding, cortex advisor composition, license gate)
- [x] `qkit_noise_filter.fl` — Observation noise filter (deduplication, rate limiting, relevance scoring, sensitivity classification, aggregation)
- [x] `qkit_sme.fl` — SME panel framework (panel registration, lifecycle management, knowledge base binding, inference rule composition)
- [x] `qkit_app_graph.fl` — App graph registry (product manifest, circuit topology, CE integration points, compliance mapping, upgrade coordination)

### Exit Criteria

- All 4 circuits compile via `estream codegen compile`
- Each circuit has `observe` + `monitor` StreamSight annotations
- Zero-linkage isolation validated (per-product HKDF contexts produce independent key material)
- Circuit count updated in `estream.toml` and `estream-component.toml` (22 → 26)

---

## Phase 3: Spec & Documentation

- [x] `specs/POLYKIT_CE_APP_GRAPH_SPEC.md` — Canonical integration specification
- [ ] `docs/CE_INTEGRATION_GUIDE.md` — Developer-facing getting started guide
- [ ] Update `CLAUDE.md` circuit count and structure references (22 → 26, add CE group)

### Exit Criteria

- Spec covers all 4 circuits, adapter pattern, noise filter profiles, SME panel lifecycle, SLG tiers
- Developer guide includes copy-paste integration template
- CLAUDE.md reflects accurate circuit inventory

---

## Phase 4: Product Integration Guides

Create per-product integration guides showing how each Tier 2 product composes the 4 CE circuits with its domain logic.

### Tier 2 Products (P0 — First Wave)

- [ ] **Q Git** — VCS-specific CE: branching strategy SME, CI policy advisor, push anomaly detection
- [ ] **Q Files** — Storage-specific CE: access pattern learning, encryption key rotation advisor, scatter-CAS optimization
- [ ] **Q Messenger** — Communication-specific CE: message pattern anomaly, relay optimization, group dynamics advisor
- [ ] **Q Mail** — Email-specific CE: phishing detection SME, SMTP bridge optimization, compliance advisor

### Tier 2 Products (P1 — Second Wave)

- [ ] **Q Docs** — Document-specific CE: collaboration pattern learning, CRDT conflict advisor, format optimization
- [ ] **Q Pass** — Security-specific CE: credential hygiene SME, breach detection, rotation policy advisor
- [ ] **Q VPN** — Network-specific CE: route optimization, traffic mimicry advisor, exit node selection
- [ ] **Q OAuth** — Identity-specific CE: auth pattern anomaly, token lifecycle advisor, SSO policy optimization

### Tier 2 Products (P2 — Third Wave)

- [ ] **Q Mind** — ESLM-specific CE: corpus curation advisor, inference quality SME, knowledge graph optimization
- [ ] **Q Wallet** — Financial-specific CE: transaction pattern learning, spend policy advisor, multi-asset optimization
- [ ] **Q Photos** — Media-specific CE: storage optimization, provenance tracking advisor, classification SME
- [ ] **Q Monitor** — Observability-specific CE: alert fatigue reduction, anomaly correlation, SLO advisor

### Exit Criteria

- Each product has a concrete FL code example showing all 4 circuit compositions
- HKDF context strings documented per product
- SME panel domains defined per product
- Noise filter profile overrides specified per product

---

## Dependencies

| Dependency | Status | Blocking |
|------------|--------|----------|
| eStream CE Phase 1 (SSM, observation ingestion) | In Progress | Phase 2 |
| eStream CE Phase 2 (cortex advisors) | Planned | Phase 4 (SME panels) |
| QKit SLG validation (`qkit_governance.fl`) | Complete | Phase 2 (license gate) |
| QKit zero-linkage (`qkit_zero_linkage.fl`) | Complete | Phase 2 (HKDF isolation) |
| QKit blind telemetry (`qkit_blind_telemetry.fl`) | Complete | Phase 2 (CE metrics) |

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| eStream CE API changes during Phase 1 | Medium | High | Abstract behind adapter; only `qkit_cognitive.fl` touches raw CE API |
| Noise filter too aggressive in production | Medium | Medium | Conservative defaults (0.6 threshold); per-product override escape hatch |
| SME panel knowledge base grows unbounded | Low | Medium | Delta-curate DAG with configurable retention windows |
| Zero-linkage constraint blocks useful cross-product CE features | Medium | Medium | Client-side bridge provides opt-in cross-product view without server-side leakage |

---

## Success Metrics

| Metric | Target |
|--------|--------|
| New CE circuits | 4 (qkit_cognitive, qkit_noise_filter, qkit_sme, qkit_app_graph) |
| Total QKit circuits | 26 |
| Products with CE integration guide | 12 (4 per wave × 3 waves) |
| Zero-linkage violations | 0 |
| CE observation noise reduction (production profile) | ≥60% reduction vs. unfiltered |
| Time-to-integrate for new product | <1 day (with guide + template) |
