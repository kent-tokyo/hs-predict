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
- **Hybrid classification pipeline** — static rule table → SMILES functional-group engine → LLM fallback (priority order)
- **Physical-form awareness** — same compound, different form = different HS code (e.g. NaOH solid → 2815.11, solution → 2815.12)
- **98-compound static rule table** — common industrial chemicals across Chapters 28, 29, 72–81
- **SMILES functional-group detection** *(v0.3)* — 20 functional groups, organic/inorganic classification, heading-level hint (≤ 0.70 confidence)
- **Mixture support** — enter each component identifier and weight fraction (w/w%) progressively
- **IUPAC name → SMILES** — auto-resolved via [`chem-name-resolver`](https://crates.io/crates/chem-name-resolver)
- **PubChem enrichment** *(v0.2, `pubchem` feature)* — fills missing identifiers from CAS / IUPAC / SMILES
- **LLM integration** *(v0.4, `llm` feature)* — **trait-hook design**: you supply any LLM client; the library supplies the prompt and result type
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
 │  Priority 1: User mapping          (confidence = 1.0)    │
 │  pipeline.with_mapping("64-19-7", "291511")              │
 └──────────────────────┬───────────────────────────────────┘
                        │ miss
                        ▼
 ┌──────────────────────────────────────────────────────────┐
 │  Priority 2: Static rule table     (98 chemicals)        │
 │  CAS + physical form + purity → exact HS subheading      │
 └──────────────────────┬───────────────────────────────────┘
                        │ miss
                        ▼
 ┌──────────────────────────────────────────────────────────┐
 │  Priority 3: SMILES functional-group engine   (v0.3)     │
 │  20 functional groups → heading-level hint (≤ 0.70)      │
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
                jp_tariff_code, recommended_action }
```

---

## SMILES functional-group detection (v0.3)

When a SMILES string is available (from the user or auto-filled by PubChem), the engine detects the following functional groups and maps them to a Chapter 29 heading hint:

| Functional group | HS heading hint | Confidence |
|---|---|---|
| Anhydride | 29.15 | 0.65 |
| Isocyanate | 29.29 | 0.70 |
| Nitrile | 29.26 | 0.70 |
| Epoxide | 29.10 | 0.70 |
| Sulphonic acid | 29.04 | 0.68 |
| Amide | 29.24 | 0.67 |
| Aldehyde | 29.12 | 0.67 |
| Ketone | 29.14 | 0.67 |
| Carboxylic acid | 29.15 | 0.60 |
| Ester | 29.15 | 0.55 |
| Phenol | 29.07 | 0.67 |
| Alcohol | 29.05 | 0.60 |
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
assert_eq!(r.heading_hint.heading, Some(2914)); // 29.14 ketone
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
| CAS number | `1310-73-2` | ✅ |
| IUPAC systematic name | `sodium hydroxide` | ✅ (fallback) |
| SMILES | `[Na+].[OH-]` | ✅ |
| InChI | `InChI=1S/Na.H2O/h;1H/q+1;/p-1` | ✅ |
| InChIKey | `HEMHJVSKTPXQMS-UHFFFAOYSA-M` | ✅ |

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
hs-predict = { version = "0.3", features = ["pubchem"] }
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

98 compounds total across Chapters 28, 29, 72–81. See [`src/rules/static_table.rs`](src/rules/static_table.rs) for the full list.

---

## Roadmap

| Version | Status | Description |
|---|---|---|
| 0.1.0 | ✅ Released | Core rule engine + Akinator session + Japan tariff codes |
| 0.2.0 | ✅ Released | PubChem API integration |
| 0.3.0 | ✅ Released | SMILES functional-group detection (20 groups, Priority 3) |
| 0.4.0 | 🔜 Planned | `LlmClassifier` trait hook + prompt builder + mock |

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
