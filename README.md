# hs-predict

[![Crates.io](https://img.shields.io/crates/v/hs-predict.svg)](https://crates.io/crates/hs-predict)
[![docs.rs](https://docs.rs/hs-predict/badge.svg)](https://docs.rs/hs-predict)
[![CI](https://github.com/kent-tokyo/hs-predict/actions/workflows/ci.yml/badge.svg)](https://github.com/kent-tokyo/hs-predict/actions)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

**HS (Harmonized System) code prediction for chemical products.**

`hs-predict` uses an **Akinator-style interactive session** — asking targeted questions one at a time — to collect just enough information to classify your product, then applies a hybrid rule-based engine to produce a six-digit HS 2022 code.

> **Disclaimer**: Predictions are advisory only and must not be used as the sole basis for a customs declaration. Always verify with a qualified trade-compliance expert or the relevant customs authority.

---

## Features

- **Akinator-style UX** — ask only what's needed; no upfront form to fill in
- **Hybrid classification pipeline** — mixture GRI → static rules → SMILES engine → LLM fallback (priority order)
- **Physical-form awareness** — same compound, different form = different HS code (e.g. NaOH solid → 2815.11, solution → 2815.12)
- **148-entry static rule table** (133 compounds) — common industrial chemicals across Chapters 28, 29, 38, 72–81
- **SMILES functional-group detection** *(v0.3)* — 20 functional groups, organic/inorganic classification, heading-level hint (≤ 0.70 confidence)
- **SMILES structural engine** *(v0.5.1)* — carbon count, hydroxyl count, ring/aromaticity/C=C detection; resolves to 6-digit HS subheading for ketones (29.14.11–31), alcohols (29.05/22.07), carboxylic acids (29.15/29.16), and aldehydes (29.12); confidence up to 0.90
- **Hydrocarbon & chloroalkane engine** *(v0.5.2)* — pure hydrocarbon detection (no heteroatoms); C=C bond counting for diene/isoprene distinction; resolves HS 2901 (acyclic: ethylene, propylene, isoprene…), 2902 (cyclic: cyclohexane, benzene, styrene…), and 2903 (chloromethane, DCM, chloroform, CCl4…) to 6-digit subheadings
- **Mixture GRI classification** *(v0.5)* — GRI 3a (same chapter), GRI 3b (essential character / dominant component > 50 % w/w), GRI 3c (last heading numerically); special-use routing for pharmaceuticals (Ch. 30), cosmetics (Ch. 33), food preparations (Ch. 21), agrochemicals (Ch. 38.08)
- **Compliance risk flags** *(v0.5)* — `GrayZone` identifies Chapter 28/29/38 boundary cases; `RecommendedAction::PriorConsultation` signals when an advance ruling (事前教示) should be requested
- **Batch processing** *(v0.5)* — `classify_batch()` and `classify_batch_with_llm()` for multi-product workflows
- **IUPAC name → SMILES** — auto-resolved via [`chem-name-resolver`](https://crates.io/crates/chem-name-resolver)
- **PubChem enrichment** *(v0.2, `pubchem` feature)* — fills missing identifiers from CAS / IUPAC / SMILES
- **LLM integration** *(v0.4, `llm` feature)* — **trait-hook design**: implement `LlmClassifier` with your HTTP client; library supplies `PromptBuilder` (EN/JA), `LlmResponse`, validation, and `MockLlmClassifier` for tests
- **Japan tariff codes** — 統計品目番号 (9-digit) included in every result, based on 実行関税率表 2026-04-01

---

## Quick start

### Interactive mode (Akinator-style)

```rust
use hs_predict::session::{ClassificationSession, Answer, SessionResult};
use hs_predict::pipeline::HsPipeline;

let mut session = ClassificationSession::new();
let pipeline = HsPipeline::new();

let q = session.start();
println!("{}", q.prompt()); // "Please enter a CAS number, IUPAC name, SMILES, or InChIKey"

match session.answer(Answer::Text("1310-73-2".to_string()))? {
    SessionResult::NeedMoreInfo { next_question } => {
        println!("{}", next_question.prompt()); // "Is this a mixture?"
    }
    SessionResult::Ready => {
        let product = session.to_product_description();
        let prediction = pipeline.classify(&product)?;
        println!("HS code: {}", prediction.display()); // "28.15.11"
        if let Some(jp) = &prediction.jp_tariff_code {
            println!("Japan tariff: {}", jp);           // "281511000"
        }
    }
    _ => {}
}
# Ok::<(), hs_predict::HsPredictError>(())
```

### Japanese session

```rust
use hs_predict::session::ClassificationSession;
use hs_predict::Language;

let mut session = ClassificationSession::new_ja(); // Japanese prompts
let q = session.start();
println!("{}", q.prompt()); // "CAS番号、IUPAC名、SMILES、InChIKey のいずれかを入力してください"
```

### Direct mode (known CAS + physical form)

```rust
use hs_predict::pipeline::HsPipeline;
use hs_predict::types::{ProductDescription, SubstanceIdentifier, PhysicalForm};

let pipeline = HsPipeline::new();

let product = ProductDescription {
    identifier: SubstanceIdentifier::from_cas("1310-73-2"), // Sodium hydroxide
    physical_form: Some(PhysicalForm::Solid),
    purity_pct: None,
    purity_type: None,
    mixture_components: None,
    intended_use: None,
    additional_context: None,
};

let p = pipeline.classify(&product)?;
assert_eq!(&p.hs_code, "281511");
assert_eq!(p.display(), "28.15.11");
# Ok::<(), hs_predict::HsPredictError>(())
```

---

## Classification pipeline

```
Input: ProductDescription
        │
        ▼
 ┌──────────────────────────────────────────────────────────┐
 │  Priority 0: Mixture GRI classifier (v0.5)               │
 │  GRI 3a → 3b (>50 % w/w) → 3c; special-use routing      │
 │  (pharmaceuticals Ch.30 / agrochemicals Ch.38.08 / …)    │
 └──────────────────────┬───────────────────────────────────┘
                        │ not a mixture
                        ▼
 ┌──────────────────────────────────────────────────────────┐
 │  Priority 1: User mapping          (confidence = 1.0)    │
 │  pipeline.with_mapping("64-19-7", "291511")              │
 └──────────────────────┬───────────────────────────────────┘
                        │ miss
                        ▼
 ┌──────────────────────────────────────────────────────────┐
 │  Priority 2: Static rule table     (133 compounds)       │
 │  CAS + physical form + purity → exact HS subheading      │
 └──────────────────────┬───────────────────────────────────┘
                        │ miss
                        ▼
 ┌──────────────────────────────────────────────────────────┐
 │  Priority 3: SMILES structural engine   (v0.5.1)          │
 │  20 functional groups + structural features               │
 │  → 6-digit subheading for ketones/acids/alcohols/ald.    │
 │  → heading hint (≤ 0.70) for other organic groups        │
 └──────────────────────┬───────────────────────────────────┘
                        │ miss / low confidence
                        ▼
 ┌──────────────────────────────────────────────────────────┐
 │  Priority 4: LLM classifier        (v0.4, trait hook)    │
 │  impl LlmClassifier for YourClient { ... }               │
 └──────────────────────┬───────────────────────────────────┘
                        │
                        ▼
                   HsPrediction
              { hs_code, confidence, notes,
                gray_zone, recommended_action,
                jp_tariff_code, alternatives }
```

---

## Mixture GRI classification (v0.5)

When `ProductDescription::mixture_components` is set, the pipeline applies the WCO General Rules for Interpretation (GRI) automatically:

| Step | Rule | Condition |
|---|---|---|
| 0 | Special-use routing | Pharmaceutical → Ch. 30; Cosmetic → Ch. 33; Food prep → Ch. 21; Agricultural → Ch. 38.08 |
| 1 | GRI 3a | All components fall in the same HS chapter → most specific heading |
| 2 | GRI 3b | One component > 50 % w/w → adopt that component's classification |
| 3 | GRI 3b LLM | No dominant component — delegate to LLM if available |
| 4 | GRI 3c | Last heading numerically; confidence 0.40; `PriorConsultation` recommended |

```rust
use hs_predict::types::{ProductDescription, SubstanceIdentifier, MixtureComponent, IntendedUse};
use hs_predict::pipeline::HsPipeline;

let pipeline = HsPipeline::new();

let product = ProductDescription {
    identifier: SubstanceIdentifier::default(),
    physical_form: None,
    purity_pct: None,
    purity_type: None,
    intended_use: Some(IntendedUse::Agricultural),
    mixture_components: Some(vec![
        MixtureComponent {
            substance: SubstanceIdentifier::from_cas("1071-83-6"), // glyphosate
            weight_fraction_pct: Some(41.0),
            volume_fraction_pct: None,
            is_solvent: false,
        },
    ]),
    additional_context: None,
};

let p = pipeline.classify(&product)?;
// Agricultural use → 3808.xx (Chapter 38)
assert_eq!(p.chapter(), "38");
# Ok::<(), hs_predict::HsPredictError>(())
```

---

## Compliance risk flags (v0.5)

`HsPrediction` now carries a `gray_zone` field to identify classification boundary risks:

```rust
use hs_predict::types::{GrayZone, RecommendedAction};

let p = pipeline.classify(&product)?;

match p.gray_zone {
    Some(GrayZone::Chapter29vs38) => {
        // Organic compound in a formulation — verify Chapter 29 vs 38
    }
    Some(GrayZone::MixtureEssentialCharacterUnclear) => {
        // GRI 3c applied — advance ruling (事前教示) strongly recommended
    }
    _ => {}
}

if p.recommended_action == RecommendedAction::PriorConsultation {
    // Contact customs authority for a binding advance ruling before declaration
}
```

| `GrayZone` variant | Meaning |
|---|---|
| `Chapter29vs38` | Organic compound may shift from Ch. 29 to Ch. 38 due to use/presentation |
| `Chapter28vs29` | Organometallic borderline — presence of metal–carbon bond is decisive |
| `MixtureEssentialCharacterUnclear` | GRI 3c applied (no dominant component); formal ruling advised |

---

## Batch processing (v0.5)

```rust
let products: Vec<ProductDescription> = vec![/* ... */];

// Synchronous batch (Priorities 0–3)
let results: Vec<Result<HsPrediction>> = pipeline.classify_batch(&products);

// Async batch with LLM fallback (Priority 4)
# #[cfg(feature = "llm")]
let results = pipeline.classify_batch_with_llm(&products).await;
```

---

## SMILES structural engine (v0.5.1)

When a SMILES string is available (from the user or auto-filled by PubChem), the engine first detects structural features (carbon count, hydroxyl groups, ring topology, C=C bonds), then resolves to a **6-digit HS subheading** for common compound classes — or falls back to a heading-level hint for others.

### 6-digit subheading resolution

| Compound class | Example | SMILES | HS subheading | Confidence |
|---|---|---|---|---|
| Acetone (3C ketone) | Acetone | `CC(C)=O` | **291411** | 0.87 |
| MEK (4C ketone) | Methyl ethyl ketone | `CCC(C)=O` | **291412** | 0.83 |
| MIBK (6C acyclic ketone) | MIBK | `CC(=O)CC(C)C` | **291413** | 0.80 |
| Cyclohexanone | Cyclohexanone | `O=C1CCCCC1` | **291422** | 0.85 |
| Acetophenone | Acetophenone | `CC(=O)c1ccccc1` | **291431** | 0.82 |
| Ethanol (2C monohydric) | Ethanol | `CCO` | **220710** | 0.85 |
| Ethylene glycol (2C diol) | Ethylene glycol | `OCCO` | **290531** | 0.85 |
| Glycerol (3C triol) | Glycerol | `OCC(O)CO` | **290541** | 0.85 |
| Acetic acid (2C) | Acetic acid | `CC(=O)O` | **291521** | 0.90 |
| Propionic acid (3C) | Propionic acid | `CCC(=O)O` | **291550** | 0.83 |
| Acrylic acid (C=C) | Acrylic acid | `OC(=O)C=C` | **291611** | 0.87 |
| Methacrylic acid | Methacrylic acid | `CC(=C)C(=O)O` | **291613** | 0.87 |
| Benzoic acid (aromatic) | Benzoic acid | `OC(=O)c1ccccc1` | **291631** | 0.85 |
| Benzaldehyde (aromatic) | Benzaldehyde | `O=Cc1ccccc1` | **291211** | 0.83 |
| Acetaldehyde (2C) | Acetaldehyde | `CC=O` | **291212** | 0.83 |

> **Note:** Ethanol routes to **Ch. 22** (undenatured ethyl alcohol ≥ 80% vol.), not Ch. 29 — consistent with WCO classification practice.

### Heading-level hints (other functional groups)

| Functional group | HS heading hint | Confidence |
|---|---|---|
| Anhydride | 29.15 | 0.65 |
| Isocyanate | 29.29 | 0.70 |
| Nitrile | 29.26 | 0.70 |
| Epoxide | 29.10 | 0.70 |
| Sulphonic acid | 29.04 | 0.68 |
| Amide | 29.24 | 0.67 |
| Ester | 29.15 | 0.55 |
| Phenol | 29.07 | 0.67 |
| Amine | 29.21 | 0.63 |
| Organohalide | 29.03 | 0.65 |
| Ether | 29.09 | 0.63 |
| Thiol / Sulphide | 29.30 | 0.65 |
| Phosphate | 29.20 | 0.62 |
| Nitro | 29.04 | 0.60 |
| Inorganic (no C–C/C–H) | Ch. 28 | 0.55 |

```rust
use hs_predict::smiles::classify_smiles;

let r = classify_smiles("CC(C)=O").unwrap(); // acetone
assert_eq!(r.heading_hint.heading, Some(2914));
assert_eq!(r.heading_hint.subheading.as_deref(), Some("291411")); // 6-digit!
assert!(r.heading_hint.confidence >= 0.85);

let r = classify_smiles("CCO").unwrap(); // ethanol → Ch. 22
assert_eq!(r.heading_hint.subheading.as_deref(), Some("220710"));
```

---

## LLM integration — design philosophy (v0.4)

### Why a trait hook, not a built-in client

HS code errors carry legal and financial consequences. Building an LLM API client directly into the library would:

- **Lock users into a specific provider** (Anthropic, OpenAI, …)
- **Create non-determinism in a compliance context** — the same compound might return different codes on different calls
- **Add secret management burden** to a library (API keys in `Cargo.toml`?)
- **Embed network latency and failure modes** into a synchronous classification call

Instead, `hs-predict` defines a trait. You implement it with whatever HTTP client, model, and prompt customisation your application needs. The library provides the structured input and validates the output.

```rust
// v0.4 — implement this trait with your preferred LLM client
use hs_predict::llm::{LlmClassifier, LlmPrompt, LlmResponse};
use futures::future::BoxFuture;

struct MyClaudeClient { api_key: String }

impl LlmClassifier for MyClaudeClient {
    fn classify<'a>(&'a self, prompt: &'a LlmPrompt) -> BoxFuture<'a, hs_predict::Result<LlmResponse>> {
        Box::pin(async move {
            // 1. Call your LLM API using prompt.system_text / prompt.user_text
            // 2. Parse the JSON response into LlmResponse
            // 3. The library validates hs_code format and chapter consistency
            todo!()
        })
    }
}

// Attach to the pipeline — no API key stored in the library
let pipeline = HsPipeline::new().with_llm(MyClaudeClient { api_key: "...".into() });
let prediction = pipeline.classify_with_llm(&product).await?;
```

The library provides:
- `LlmPrompt` — pre-built system prompt + user message (product info + SMILES hints)
- `LlmResponse` — the expected return type (`hs_code`, `confidence`, `rationale`, `alternatives`)
- Chapter-consistency validation (LLM code vs. SMILES engine hint)
- `MockLlmClassifier` under the `mock` feature for testing

---

## PubChem enrichment (v0.2)

PubChem integration fills in missing identifier fields before classification.
It is **factual data retrieval** (deterministic), not classification — a different role from the LLM fallback.

```rust
# #[cfg(feature = "pubchem")]
# async fn example() -> hs_predict::Result<()> {
use hs_predict::pipeline::HsPipeline;
use hs_predict::pubchem::PubChemClient;
use hs_predict::types::{ProductDescription, SubstanceIdentifier, PhysicalForm};

let pipeline = HsPipeline::new().with_pubchem(PubChemClient::new());

let mut product = ProductDescription {
    identifier: SubstanceIdentifier::from_cas("1310-73-2"),
    physical_form: Some(PhysicalForm::Solid),
    purity_pct: None,
    purity_type: None,
    mixture_components: None,
    intended_use: None,
    additional_context: None,
};

pipeline.enrich(&mut product).await?;  // fills SMILES, InChI, IUPAC name …
let prediction = pipeline.classify(&product)?;
println!("{}", prediction.display()); // "28.15.11"
# Ok(())
# }
```

---

## Akinator question flow

```
Q1: CAS / IUPAC name / SMILES / InChIKey?
     │
     ├─ PubChem lookup (pubchem feature) ────────────────────────────┐
     │                                                                │
     ▼                                                                ▼
Q2: Is this a mixture?
     │
     ├─ Yes ──► Q: How many components?
     │               └─ For each component:
     │                   ├─ Q: Identifier?
     │                   └─ Q: Weight fraction (w/w%)?
     │
     └─ No ───► Q3: Physical form?
                    (Solid / Powder / Granules / Liquid /
                     Solution / Gas / Foil / Ingot / Unknown)
                     │
                     ├─ Solution ──► Q: Concentration (w/w%)?
                     │
                     ▼
                Q4: Intended use?
                    (Industrial / Pharmaceutical / Agricultural /
                     Food / Cosmetic / Other)
                     │
                     ├─ No SMILES ──► Q5: Organic or Inorganic?
                     │                     │
                     │                     └─ Organic ──► Q6: Functional groups?
                     ▼
                 Classification pipeline (Priorities 1–4)
```

---

## Supported identifiers

| Format | Example | Auto-detected |
|---|---|---|
| CAS number | `1310-73-2` | yes |
| IUPAC systematic name | `sodium hydroxide` | yes (fallback) |
| SMILES | `[Na+].[OH-]` | yes |
| InChI | `InChI=1S/Na.H2O/h;1H/q+1;/p-1` | yes |
| InChIKey | `HEMHJVSKTPXQMS-UHFFFAOYSA-M` | yes |

> Only IUPAC systematic names are accepted as text input. Trade names and common aliases (e.g. "caustic soda") are not supported — they cannot be reliably resolved.

---

## Feature flags

| Flag | Enables | Extra dependencies |
|---|---|---|
| *(none)* | Rule-based + SMILES engine (Priorities 1–3) | — |
| `pubchem` | PubChem identifier enrichment | `reqwest`, `moka`, `governor` |
| `llm` | `LlmClassifier` trait + pipeline Priority 4 | — |
| `mock` | `MockLlmClassifier` for unit testing | — |

```toml
[dependencies]
hs-predict = { version = "0.5.2", features = ["pubchem"] }
```

---

## Example chemicals (static rule table)

| CAS | Substance | Form | HS 2022 |
|---|---|---|---|
| 1310-73-2 | Sodium hydroxide | Solid | 2815.11 |
| 1310-73-2 | Sodium hydroxide | Solution | 2815.12 |
| 7664-93-9 | Sulphuric acid | Any | 2807.00 |
| 7697-37-2 | Nitric acid | ≥ 98% | 2808.10 |
| 7697-37-2 | Nitric acid | < 98% | 2808.90 |
| 7664-41-7 | Ammonia | Gas | 2814.10 |
| 7664-41-7 | Ammonia | Solution | 2814.20 |
| 7429-90-5 | Aluminium | Ingot ≥ 99% | 7601.10 |
| 7429-90-5 | Aluminium | Powder | 7603.10 |
| 7429-90-5 | Aluminium | Foil | 7607.11 |
| 67-56-1 | Methanol | Liquid | 2905.11 |
| 64-17-5 | Ethanol | Liquid | 2207.10 |
| 67-64-1 | Acetone | Liquid | 2914.11 |

133 compounds (148 rule entries) across Chapters 28, 29, 38, 72–81. See [`src/rules/static_table.rs`](src/rules/static_table.rs) for the full list.

---

## Roadmap

| Version | Status | Description |
|---|---|---|
| 0.1.0 | Released | Core rule engine + Akinator session + Japan tariff codes |
| 0.2.0 | Released | PubChem API integration |
| 0.3.0 | Released | SMILES functional-group detection (20 groups, Priority 3) |
| 0.4.0 | Released | `LlmClassifier` trait hook + `PromptBuilder` (EN/JA) + `MockLlmClassifier` + WASM |
| 0.4.1 | Released | WASM companion crate + `Serialize` additions |
| 0.5.0 | Released | Mixture GRI 3a/3b/3c · `GrayZone` · `PriorConsultation` · 133 compounds · batch · security hardening |
| 0.5.1 | Released | SMILES structural engine — 6-digit subheading for ketones, alcohols, acids, aldehydes; confidence up to 0.90 |
| 0.5.2 | Released | Hydrocarbon engine (HS 2901/2902) + chloroalkane engine (HS 2903) — isoprene, cyclohexane, DCM and 13 more CAS entries |
| 0.6.0 | Planned | npm publish · GitHub Actions CI · WASM tests |

---

## Minimum Supported Rust Version (MSRV)

Rust **1.75**.

---

## Contributing

Bug reports, rule-table additions, and PRs are welcome.  
For new entries in the static rule table, please cite the HS 2022 nomenclature chapter/note that supports the classification.

---

## License

Licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

at your option.
