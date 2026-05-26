# hs-predict-wasm

[![Crates.io](https://img.shields.io/crates/v/hs-predict-wasm.svg)](https://crates.io/crates/hs-predict-wasm)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

**WebAssembly bindings for [hs-predict](https://crates.io/crates/hs-predict) — HS code prediction for chemical products.**

Exposes the full `hs-predict` classification engine to JavaScript / TypeScript via `wasm-pack`.

---

## Build

```bash
cd hs-predict-wasm
wasm-pack build --target web --release
# Generates ./pkg/  (~320 KB)
```

For Node.js:
```bash
wasm-pack build --target nodejs --release
```

---

## APIs

### 1. `classify_smiles(smiles: string): object | null`

Analyse a SMILES string and return functional-group detection and an HS heading hint.

```js
import init, { classify_smiles } from './pkg/hs_predict_wasm.js';
await init();

const r = classify_smiles('CC(C)=O');  // acetone
// {
//   organic_class: "organic",
//   functional_groups: ["Ketone"],
//   structural_features: {
//     carbon_count: 3, hydroxyl_count: 0, carbonyl_count: 1,
//     has_ring: false, has_aromatic_ring: false,
//     has_cc_double_bond: false, cc_double_bond_count: 0,
//     is_pure_hydrocarbon: false, ...
//   },
//   heading_hint: {
//     chapter: 29, heading: 2914,
//     subheading: "291411",       // 6-digit when determinable
//     confidence: 0.87,
//     rationale: "acetone"
//   }
// }

const r2 = classify_smiles('C=CC(C)=C');  // isoprene
// heading_hint: { chapter: 29, heading: 2901, subheading: "290124", confidence: 0.87 }

const r3 = classify_smiles('ClCCl');  // dichloromethane
// heading_hint: { chapter: 29, heading: 2903, subheading: "290312", confidence: 0.90 }

classify_smiles('');  // null
```

Returns `null` for empty or whitespace-only input (max 4096 bytes).

---

### 2. `classify_product(product_json: string): object`

Run the full rule-based pipeline (mixture GRI → static rules → SMILES engine) on a serialised `ProductDescription`.

```js
import init, { classify_product } from './pkg/hs_predict_wasm.js';
await init();

const pred = classify_product(JSON.stringify({
  identifier: { cas: "1310-73-2" },   // Sodium hydroxide
  physical_form: "Solid",
  purity_pct: null,
  purity_type: null,
  mixture_components: null,
  intended_use: null,
  additional_context: null
}));
// {
//   hs_code: "281511",
//   confidence: 1.0,
//   heading_description: "Sodium hydroxide, solid",
//   recommended_action: "Accept",
//   gray_zone: null,
//   jp_tariff_code: "281511000",
//   alternatives: []
// }
```

Throws a string error on invalid JSON input or classification failure.

#### `ProductDescription` schema

```ts
interface ProductDescription {
  identifier: {
    cas?: string;          // e.g. "67-64-1"
    smiles?: string;       // e.g. "CC(C)=O"
    iupac_name?: string;   // e.g. "propan-2-one"
    inchi?: string;
    inchi_key?: string;
    cid?: number;          // PubChem CID
  };
  physical_form?: "Solid" | "Powder" | "Granules" | "Liquid" |
                  "Solution" | "Gas" | "Foil" | "Ingot" | "Unknown";
  purity_pct?: number;
  purity_type?: "WeightPercent" | "VolumePercent" | "MolePercent";
  mixture_components?: MixtureComponent[];
  intended_use?: "Industrial" | "Pharmaceutical" | "Agricultural" |
                 "Food" | "Cosmetic" | "Other";
  additional_context?: string;
}

interface MixtureComponent {
  substance: ProductDescription["identifier"];
  weight_fraction_pct?: number;
  volume_fraction_pct?: number;
  is_solvent: boolean;
}
```

---

### 3. `WasmSession` — Akinator-style interactive session

Step-by-step session that asks only what is needed to classify the product.

```js
import init, { WasmSession } from './pkg/hs_predict_wasm.js';
await init();

// English session (default)
const session = new WasmSession();

// Japanese session
// const session = WasmSession.new_ja();

// Step 1: start
const q1 = session.start();
// { step: "Identifier", prompt: "Please enter a CAS number...", type: "text", choices: null }

// Step 2: answer questions in a loop
let result = session.answer(JSON.stringify({ Text: "1310-73-2" }));
// { type: "NeedMoreInfo", next_question: { step: "IsMixture", prompt: "Is this a mixture?", type: "yes_no" } }

result = session.answer(JSON.stringify({ YesNo: false }));
// { type: "NeedMoreInfo", next_question: { step: "PhysicalForm", ... } }

result = session.answer(JSON.stringify({ Choice: 0 }));  // "Solid"
// { type: "Ready" }

// Step 3: classify when Ready
const prediction = session.classify();
// { hs_code: "281511", confidence: 1.0, jp_tariff_code: "281511000", ... }
```

#### Answer variants

| Answer JSON | Use when |
|---|---|
| `{ "Text": "67-64-1" }` | Free-text question (identifier, context) |
| `{ "YesNo": true }` | Yes/No question |
| `{ "Choice": 2 }` | Single-choice question (0-indexed) |
| `{ "MultiChoice": [0, 2] }` | Multi-choice question |
| `{ "Number": 99.5 }` | Numeric question (purity, concentration) |

#### Session result variants

| `type` | Meaning |
|---|---|
| `"NeedMoreInfo"` | More questions remain; `next_question` is set |
| `"Ready"` | All information collected; call `session.classify()` |
| `"RequiresLlm"` | Rule engine insufficient; LLM fallback needed (not available in WASM) |

---

## Classification pipeline (Priorities 0–3)

| Priority | Method | Confidence |
|---|---|---|
| 0 | Mixture GRI 3a/3b/3c + special-use routing | varies |
| 1 | User-provided mapping | 1.0 |
| 2 | Static rule table (161 CAS entries) | 0.95–1.0 |
| 3 | SMILES structural engine | 0.60–0.90 |

Note: LLM fallback (Priority 4) requires a network-capable client and is not included in the WASM bundle.

---

## Feature notes

- **No async required** — the WASM bundle uses only the rule-based engine (Priorities 0–3); no HTTP requests are made.
- **No WASI** — targets `wasm32-unknown-unknown`; runs in browsers and Node.js.
- **Bundle size** — approximately 320 KB (release build, `opt-level = "z"`, LTO).

---

## Compatibility

| Tool | Version |
|---|---|
| wasm-pack | 0.13+ |
| wasm-bindgen | 0.2.121 (pinned) |
| Rust | 1.75+ |

---

## Related

- [hs-predict](https://crates.io/crates/hs-predict) — the core Rust library (async, LLM, PubChem)
- [Repository](https://github.com/kent-tokyo/hs-predict)

---

## License

Licensed under either of [MIT License](../LICENSE-MIT) or [Apache License 2.0](../LICENSE-APACHE) at your option.
