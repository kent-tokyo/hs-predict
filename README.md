# hs-predict

[![Crates.io](https://img.shields.io/crates/v/hs-predict.svg)](https://crates.io/crates/hs-predict)
[![docs.rs](https://docs.rs/hs-predict/badge.svg)](https://docs.rs/hs-predict)
[![CI](https://github.com/kent-tokyo/hs-predict/actions/workflows/ci.yml/badge.svg)](https://github.com/kent-tokyo/hs-predict/actions)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

**HS (Harmonized System) code prediction for chemical products.**

`hs-predict` uses an **Akinator-style interactive session** — asking targeted questions one at a time — to collect just enough information to classify your product, then applies a hybrid rule-based and LLM engine to produce a six-digit HS 2022 code.

> **Disclaimer**: Predictions are advisory only and must not be used as the sole basis for a customs declaration. Always verify with a qualified trade-compliance expert or the relevant customs authority.

---

## Features

- **Akinator-style UX** — ask only what's needed to narrow down the HS code; no upfront form to fill
- **Hybrid pipeline** — static rule table → SMILES rule engine → LLM fallback (priority order)
- **Physical-form awareness** — same compound, different form = different HS code (e.g. NaOH solid → 2815.11, solution → 2815.12)
- **Mixture support** — enter each component identifier and weight fraction (w/w%) progressively
- **IUPAC name → SMILES** — auto-resolved via [`chem-name-resolver`](https://crates.io/crates/chem-name-resolver)
- **PubChem integration** *(v0.2, `pubchem` feature)* — auto-enrichment from CAS / IUPAC / SMILES
- **LLM integration** *(v0.4, `llm` feature)* — Claude / OpenAI-compatible API with chapter-consistency validation

---

## Quick start

### Interactive mode (Akinator-style)

```rust
use hs_predict::session::{ClassificationSession, Answer, SessionResult};
use hs_predict::pipeline::HsPipeline;

let mut session = ClassificationSession::new();
let pipeline = HsPipeline::new();

// Start the session — receive the first question
let q = session.start();
println!("{}", q.prompt());
// "CAS番号、IUPAC名、SMILES、InChIKey のいずれかを入力してください"

// Answer with a CAS number (auto-detected format)
match session.answer(Answer::Text("1310-73-2".to_string()))? {
    SessionResult::NeedMoreInfo { next_question } => {
        println!("Next: {}", next_question.prompt());
        // "混合物ですか？"
    }
    SessionResult::Ready => {
        let product = session.to_product_description();
        let prediction = pipeline.classify(&product)?;
        println!("HS code: {}", prediction.display()); // "28.15.11"
        println!("Confidence: {:.0}%", prediction.confidence * 100.0);
    }
    SessionResult::RequiresLlm => {
        // Enable the `llm` feature and configure an API key for this path
    }
}
# Ok::<(), hs_predict::HsPredictError>(())
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

## Akinator flow

```
Q1: CAS number / IUPAC name / SMILES / InChIKey?
     │
     ├─ PubChem lookup (v0.2) ─────────────────────────────────┐
     │                                                           │
     ▼                                                           ▼
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
                     Food / Cosmetic / Research / Other)
                     │
                     ├─ No SMILES ──► Q5: Organic / Inorganic?
                     │                     │
                     │                     └─ Organic ──► Q6: Functional groups?
                     ▼
                 ┌─────────────────────────────────────────────┐
                 │            Hybrid pipeline                  │
                 │  1. User mapping  (confidence = 1.0)        │
                 │  2. Static rule table (~50 chemicals)       │
                 │  3. SMILES rule engine   (v0.3)             │
                 │  4. LLM API              (v0.4)             │
                 └─────────────────────────────────────────────┘
                              │
                              ▼
                         HsPrediction
                          hs_code, confidence, notes, alternatives
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

> **Note**: Only IUPAC systematic names are accepted as text input. Trade names and common aliases (e.g. "caustic soda", "lye") are not supported — they cannot be reliably resolved.

---

## Feature flags

| Flag | Description | Dependencies added |
|---|---|---|
| *(none)* | Rule-based engine only | — |
| `pubchem` | PubChem API enrichment (v0.2) | `reqwest`, `moka`, `governor` |
| `llm` | LLM API integration (v0.4) | `reqwest`, `tokio` |
| `mock` | Mock LLM client for testing | — |

Add to `Cargo.toml`:

```toml
[dependencies]
hs-predict = { version = "0.1", features = ["pubchem", "llm"] }
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

---

## Minimum Supported Rust Version (MSRV)

Rust **1.75** (stabilised `async fn` in traits).

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
