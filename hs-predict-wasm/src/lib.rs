//! WebAssembly bindings for `hs-predict`.
//!
//! Exposes three APIs:
//!
//! - **`classify_smiles(smiles)`** — SMILES → functional groups + HS heading hint
//! - **`classify_product(product_json)`** — full rule-based pipeline (Priorities 1–3)
//! - **`WasmSession`** — Akinator-style interactive session
//!
//! Build with:
//! ```bash
//! wasm-pack build --target web --release
//! ```
//!
//! # JavaScript usage
//! ```js
//! import init, { classify_smiles, WasmSession } from './pkg/hs_predict_wasm.js';
//! await init();
//!
//! // 1. SMILES classification
//! const r = classify_smiles('CC(O)=O');
//! // → { organic_class: "organic", functional_groups: ["CarboxylicAcid"],
//! //     heading_hint: { chapter: 29, heading: 2915, confidence: 0.6, rationale: "..." } }
//!
//! // 2. Rule-based pipeline
//! const pred = classify_product(JSON.stringify({
//!   identifier: { cas: "1310-73-2" },
//!   physical_form: "Solid"
//! }));
//! // → { hs_code: "281511", confidence: 1.0, ... }
//!
//! // 3. Interactive session
//! const session = new WasmSession();
//! const q1 = session.start();
//! // → { step: "Identifier", prompt: "Please enter a CAS number...", type: "text" }
//! const r1 = session.answer(JSON.stringify({ Text: "1310-73-2" }));
//! // → { type: "NeedMoreInfo", next_question: { step: "IsMixture", ... } }
//! ```

use wasm_bindgen::prelude::*;

use hs_predict::pipeline::HsPipeline;
use hs_predict::session::{Answer, ClassificationSession};
use hs_predict::types::ProductDescription;

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

fn to_js<T: serde::Serialize>(val: &T) -> JsValue {
    serde_wasm_bindgen::to_value(val).unwrap_or(JsValue::NULL)
}

fn err_js(e: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// API 1 — SMILES classification
// ─────────────────────────────────────────────────────────────────────────────

/// Analyse a SMILES string and return functional-group + HS heading hint.
///
/// Returns a `SmilesClassification` JS object, or `null` if the SMILES string
/// is empty or cannot be parsed.
///
/// # JS return shape
/// ```json
/// {
///   "organic_class": "organic",
///   "functional_groups": ["CarboxylicAcid"],
///   "heading_hint": {
///     "chapter": 29,
///     "heading": 2915,
///     "rationale": "Carboxylic acid → heading 29.15",
///     "confidence": 0.60
///   }
/// }
/// ```
#[wasm_bindgen]
pub fn classify_smiles(smiles: &str) -> JsValue {
    match hs_predict::smiles::classify_smiles(smiles) {
        Some(r) => to_js(&r),
        None => JsValue::NULL,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// API 2 — Full rule-based pipeline
// ─────────────────────────────────────────────────────────────────────────────

/// Classify a chemical product using the full rule-based pipeline (Priorities 1–3).
///
/// `product_json` must be a JSON-serialised `ProductDescription`:
/// ```json
/// {
///   "identifier": { "cas": "1310-73-2" },
///   "physical_form": "Solid",
///   "purity_pct": null,
///   "purity_type": null,
///   "mixture_components": null,
///   "intended_use": null,
///   "additional_context": null
/// }
/// ```
///
/// Returns a `HsPrediction` JS object on success, or throws a JS error string.
#[wasm_bindgen]
pub fn classify_product(product_json: &str) -> Result<JsValue, JsValue> {
    let product: ProductDescription =
        serde_json::from_str(product_json).map_err(|e| err_js(e))?;
    let pipeline = HsPipeline::new();
    pipeline
        .classify(&product)
        .map(|pred| to_js(&pred))
        .map_err(|e| err_js(e))
}

// ─────────────────────────────────────────────────────────────────────────────
// API 3 — Interactive Akinator session
// ─────────────────────────────────────────────────────────────────────────────

/// Interactive Akinator-style HS classification session.
///
/// # Typical flow
/// ```js
/// const session = new WasmSession();       // English
/// // const session = WasmSession.new_ja(); // Japanese
///
/// const q1 = session.start();
/// // q1 = { step: "Identifier", prompt: "...", type: "text", choices: null, ... }
///
/// const r1 = session.answer(JSON.stringify({ Text: "64-19-7" }));
/// // r1 = { type: "NeedMoreInfo", next_question: { step: "IsMixture", ... } }
///
/// // … repeat until r.type === "Ready" …
///
/// const prediction = session.classify();
/// // { hs_code: "291511", confidence: 0.95, ... }
/// ```
#[wasm_bindgen]
pub struct WasmSession {
    session: ClassificationSession,
    pipeline: HsPipeline,
}

#[wasm_bindgen]
impl WasmSession {
    /// Create a new session with English prompts.
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmSession {
        WasmSession {
            session: ClassificationSession::new(),
            pipeline: HsPipeline::new(),
        }
    }

    /// Create a new session with Japanese prompts.
    pub fn new_ja() -> WasmSession {
        WasmSession {
            session: ClassificationSession::new_ja(),
            pipeline: HsPipeline::new(),
        }
    }

    /// Start the session and return the first `Question` as a JS object.
    ///
    /// # JS return shape
    /// ```json
    /// {
    ///   "step": "Identifier",
    ///   "prompt": "Please enter a CAS number, IUPAC name, SMILES, or InChIKey",
    ///   "type": "text",
    ///   "choices": null,
    ///   "number_range": null
    /// }
    /// ```
    pub fn start(&mut self) -> JsValue {
        let q = self.session.start();
        to_js(&q)
    }

    /// Submit an answer and advance the session.
    ///
    /// `answer_json` must be the JSON representation of an `Answer` variant:
    /// - `JSON.stringify({ Text: "1310-73-2" })` — free-text answer
    /// - `JSON.stringify({ YesNo: true })` — yes/no answer
    /// - `JSON.stringify({ Choice: 0 })` — single-choice index
    /// - `JSON.stringify({ MultiChoice: [0, 2] })` — multi-choice indices
    /// - `JSON.stringify({ Number: 30.5 })` — numeric answer
    ///
    /// # JS return shape
    /// ```json
    /// { "type": "NeedMoreInfo", "next_question": { "step": "...", ... } }
    /// { "type": "Ready" }
    /// { "type": "RequiresLlm" }
    /// ```
    pub fn answer(&mut self, answer_json: &str) -> Result<JsValue, JsValue> {
        let answer: Answer = serde_json::from_str(answer_json).map_err(|e| err_js(e))?;
        self.session
            .answer(answer)
            .map(|result| to_js(&result))
            .map_err(|e| err_js(e))
    }

    /// Classify the product once the session is `Ready`.
    ///
    /// Call this after `answer()` returns `{ type: "Ready" }`.
    ///
    /// Returns a `HsPrediction` JS object on success, or throws a JS error string.
    pub fn classify(&self) -> Result<JsValue, JsValue> {
        let product = self.session.to_product_description();
        self.pipeline
            .classify(&product)
            .map(|pred| to_js(&pred))
            .map_err(|e| err_js(e))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WASM tests (run with: wasm-pack test --node)
// ─────────────────────────────────────────────────────────────────────────────
//
// These tests run in a JS/WASM environment only.
// Core classification logic is already covered by hs-predict's own test suite.

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn classify_smiles_acetic_acid_returns_non_null() {
        let r = classify_smiles("CC(O)=O");
        assert!(!r.is_null(), "acetic acid SMILES should return a classification");
    }

    #[wasm_bindgen_test]
    fn classify_smiles_empty_returns_null() {
        let r = classify_smiles("");
        assert!(r.is_null(), "empty SMILES should return null");
    }

    #[wasm_bindgen_test]
    fn classify_product_naoh_solid_ok() {
        let json = r#"{
            "identifier": { "cas": "1310-73-2", "smiles": null, "iupac_name": null,
                            "inchi": null, "inchi_key": null, "cid": null },
            "physical_form": "Solid",
            "purity_pct": null,
            "purity_type": null,
            "mixture_components": null,
            "intended_use": null,
            "additional_context": null
        }"#;
        let r = classify_product(json);
        assert!(r.is_ok(), "NaOH solid should classify successfully");
    }

    #[wasm_bindgen_test]
    fn classify_product_invalid_json_returns_err() {
        let r = classify_product("not json at all");
        assert!(r.is_err());
    }

    #[wasm_bindgen_test]
    fn wasm_session_start_returns_question() {
        let mut s = WasmSession::new();
        let q = s.start();
        assert!(!q.is_null(), "start() should return a question");
    }

    #[wasm_bindgen_test]
    fn wasm_session_answer_valid_cas() {
        let mut s = WasmSession::new();
        s.start();
        let r = s.answer(r#"{"Text":"1310-73-2"}"#);
        assert!(r.is_ok(), "valid CAS should be accepted");
    }

    #[wasm_bindgen_test]
    fn wasm_session_answer_invalid_json_returns_err() {
        let mut s = WasmSession::new();
        s.start();
        let r = s.answer("not json");
        assert!(r.is_err());
    }
}
