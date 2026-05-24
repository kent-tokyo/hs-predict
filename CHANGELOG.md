# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.4.1] — 2026-05-24

### Added

#### WebAssembly support (`hs-predict-wasm` workspace crate)

- New companion crate **`hs-predict-wasm`** (published separately on crates.io) exposes
  the full classification engine to JavaScript via `wasm-bindgen`:
  - `classify_smiles(smiles: &str) -> JsValue` — SMILES → `SmilesClassification` JS object
  - `classify_product(json: &str) -> Result<JsValue, JsValue>` — full rule-based pipeline
  - `WasmSession` — Akinator-style interactive session (`new()` / `new_ja()` / `start()` /
    `answer()` / `classify()`)
  - Build: `wasm-pack build --target web --release` → `pkg/` (~317 KB `.wasm`)

- **`SmilesClassification`**, **`HeadingHint`**, **`SessionResult`** now derive
  `serde::Serialize` (required for `serde-wasm-bindgen` serialisation to JS objects).
  `SessionResult` uses `#[serde(tag = "type")]` so JS receives `{ type: "Ready" }` etc.

#### Workspace

- Root `Cargo.toml` converted to a Cargo workspace (`members = [".", "hs-predict-wasm"]`).
- `[profile.release]` (`opt-level = "z"`, `lto = true`, `panic = "abort"`) moved to
  workspace root so the WASM optimisation settings apply uniformly.

### Notes

- No breaking changes to the `hs-predict` public API.
- `hs-predict-wasm` requires **wasm-pack** and the `wasm32-unknown-unknown` target;
  it is not needed for standard Rust / server-side use.

---

## [0.4.0] — 2026-05-24

### Added

#### LLM trait hook (`llm` feature)

- **`LlmClassifier`** trait — implement this with any HTTP client / model:
  ```rust
  pub trait LlmClassifier: Send + Sync {
      fn classify<'a>(&'a self, prompt: &'a LlmPrompt)
          -> BoxFuture<'a, Result<LlmResponse>>;
  }
  ```
  Uses `BoxFuture` from `futures` for object safety; no `async_trait` macro needed.

- **`LlmPrompt`** — pre-built system + user text (EN/JA) plus optional
  `smiles_analysis: Option<SmilesClassification>` for custom prompt construction.

- **`LlmResponse`** + **`LlmAlternative`** — typed return value with
  `hs_code`, `confidence`, `rationale`, and `alternatives` (serde).

- **`parse_llm_json`** — helper that strips markdown fences
  (` ```json `, ` ``` `) and deserialises the LLM's JSON reply into `LlmResponse`.

- **`PromptBuilder`** (`llm` feature) — converts a `ProductDescription`
  into a ready-to-send `LlmPrompt`.
  - EN and JA system prompts (role, output schema, confidence guide, rules)
  - User message includes all identifiers, physical form, purity, mixture
    components, and SMILES functional-group hints when available.
  - `PromptBuilder::new()` / `PromptBuilder::with_language(Language)`

- **`MockLlmClassifier`** (`mock` feature, implies `llm`) — deterministic
  stub for unit tests. Derives HS code from SMILES analysis when available;
  falls back to a configurable default code (`"999999"` by default). No
  network call ever made.
  - `MockLlmClassifier::new()` — default fallback `"999999"`
  - `MockLlmClassifier::with_default(hs_code, confidence)` — custom fallback

#### Pipeline — Priority 4 (`llm` feature)

- **`HsPipeline::with_llm(client)`** — attach any `impl LlmClassifier`
  (stored as `Arc<dyn LlmClassifier>`).

- **`HsPipeline::classify_with_llm(&product)`** — async classification that
  first runs Priorities 1–3 and calls the LLM only when the result has
  `recommended_action != Accept` or returns `LowConfidenceNoLlm`.

  Validation:
  - LLM `hs_code` must be exactly 6 ASCII digits →
    `HsPredictError::ValidationFailed` otherwise.
  - Chapter mismatch vs. SMILES hint appends a warning note (not a hard error).

#### Feature flags

- `llm = []` — **no new compile-time dependencies** (removed `reqwest`/`tokio`
  from the `llm` feature; the trait hook design requires no network code in
  the library).
- `mock = ["llm"]` — implies `llm`.

### Tests

- 5 new pipeline integration tests (`pipeline::tests`, `mock` feature):
  - `classify_with_llm_mock_returns_6_digit_code`
  - `classify_with_llm_mock_chapter_29_for_smiles_acid`
  - `classify_with_llm_no_client_returns_error`
  - `classify_with_llm_skips_llm_for_high_confidence_rule`
  - `classify_with_llm_invalid_code_returns_validation_error`
- 5 mock tests (`llm::mock::tests`)
- 9 prompt-builder tests (`llm::prompt::tests`)

---

## [0.3.0] — 2026-05-24

### Added

#### SMILES functional-group detection engine (`smiles` module)
- `FunctionalGroup` — 20-variant enum covering the main HS Chapter 29 criteria:  
  `Anhydride`, `Isocyanate`, `Nitrile`, `Nitro`, `Epoxide`, `SulphonicAcid`,  
  `Phosphate`, `Amide`, `Ester`, `CarboxylicAcid`, `Aldehyde`, `Ketone`,  
  `Phenol`, `Thiol`, `Sulphide`, `Alcohol`, `Ether`, `Amine`, `Halide`, `AromaticRing`
- `classify_organic(&str) -> OrganicInorganic` — distinguishes organic / inorganic /
  organometallic from a SMILES string (pattern-based, no external dependencies)
- `detect_functional_groups(&str) -> Vec<FunctionalGroup>` — priority-ordered detection
  using substring matching against PubChem canonical SMILES
- `HeadingHint` — chapter + 4-digit heading hint with confidence and rationale string
- `map_to_heading(&[FunctionalGroup], &OrganicInorganic) -> HeadingHint` — maps detected
  groups to HS chapter/heading via a 20-entry priority table
- `classify_smiles(&str) -> Option<SmilesClassification>` — public entry point that
  combines organic classification + group detection + heading mapping
- `SmilesClassification` — result struct with `organic_class`, `functional_groups`, `heading_hint`

#### Pipeline — Priority 3 integration
- `HsPipeline::classify` now tries SMILES-based classification when the static rule table
  finds no match (Priority 3), using the detected heading as a 4-digit hint padded to
  six digits (`XXXX00`) with `RecommendedAction::VerifyWithLlm`

#### Bug fix — docs.rs failure (v0.2.0 regression)
- Added stub `src/llm/mod.rs`; the empty `src/llm/` directory caused docs.rs to fail
  to compile the crate with `--all-features`

### Tests
- 35 new unit tests (all passing):
  - `smiles::detector`: 19 tests (organic/inorganic detection + all 20 functional groups)
  - `smiles::chapter_map`: 11 tests (inorganic Ch.28, organometallic 29.31, heading priority)
  - `smiles::mod`: 10 tests (integration: SMILES → heading mapping end-to-end)

---

## [0.2.0] — 2026-05-24

### Added

#### PubChem API integration (`pubchem` feature)
- `PubChemClient` — async HTTP client with built-in rate limiting (5 req/s, configurable) and in-memory caching (1 000 entries, 24 h TTL)
- `PubChemClientBuilder` — builder for custom rate limits, cache size/TTL, and base URL (useful for testing against a local mock server)
- `PubChemCompound` — structured result containing CID, IUPAC name, canonical SMILES, InChI, InChIKey, molecular formula, and molecular weight
- `PubChemClient::lookup(&SubstanceIdentifier)` — look up by CAS number, InChIKey, InChI, SMILES, or IUPAC name (priority order)
- `PubChemClient::enrich(&mut SubstanceIdentifier)` — fills missing identifier fields in place (best-effort; NotFound is silently ignored)
- `HsPipeline::with_pubchem(PubChemClient)` — attach a client to the pipeline
- `HsPipeline::enrich(&mut ProductDescription)` — async enrichment of main identifier + all mixture components
- 7 unit tests for PubChem module (2 integration tests gated behind `--ignored`, require internet)

---

## [0.1.0] — 2026-05-24

### Added

#### Core types
- `SubstanceIdentifier` — unified identifier accepting CAS number, SMILES, IUPAC name, InChI, and InChIKey
- `PhysicalForm` — shape variants: solid, powder, granules, liquid, solution (with concentration), gas, foil, ingot
- `MixtureComponent` — component descriptor with optional weight fraction (w/w%)
- `Language` — locale selector (`Language::En` / `Language::Ja`) for session prompts
- `HsPrediction` — prediction result with `jp_tariff_code` and `jp_tariff_year` fields for Japan customs

#### Classification pipeline
- `HsPipeline` — hybrid classification pipeline (rule-based priority; LLM/PubChem as future fallback)
- `HsPipeline::with_mapping()` — user-supplied CAS → HS code overrides (confidence = 1.0)
- `HsPipeline::with_config()` — configurable confidence thresholds

#### Static rule table (`rules::static_table`)
- 98 CAS → HS 2022 rules covering common industrial chemicals:
  - Chapter 28: inorganic acids/bases/salts/oxides (NaOH, H₂SO₄, HCl, HNO₃, H₃PO₄, HF, NH₃, KOH, and 30+ more)
  - Chapter 29: organic solvents, acids, and intermediates (MeOH, EtOH, IPA, acetone, toluene, benzene, DCM, CHCl₃, DMSO, DMF, and 20+ more)
  - Chapters 72–81: aluminium, copper, zinc, nickel, lead, tin, silver (with shape-specific sub-rules)
- Shape/purity-aware matching with specificity scoring (more specific rules always win)

#### Japan tariff codes (`rules::jp_table`)
- `find_jp_rule()` — look up 統計品目番号 (9-digit Japan statistical item code) from HS heading
- ~70 entries covering all HS codes in the static rule table
- Based on 実行関税率表 2026-04-01 revision
- `JP_TARIFF_YEAR` constant for transparency

#### Interactive session (`session`)
- `ClassificationSession` — Akinator-style interactive session
- `QuestionStep` — language-independent step enum for dispatch (no prompt-text matching)
- `session::messages` — all user-facing strings centralised for easy localisation
- Multilingual prompts: English (default) and Japanese
  - `ClassificationSession::new()` — English prompts
  - `ClassificationSession::new_ja()` — Japanese prompts
  - `ClassificationSession::with_language(Language)` — builder-style override
- Session serialisation / deserialisation (Serde) for pause-and-resume
- Automatic IUPAC name → SMILES resolution via `chem-name-resolver`
- Question types: `Text`, `YesNo`, `Choice`, `MultiChoice`, `Number`

#### Feature flags
- `pubchem` — HTTP client + cache + rate limiter (implementation in v0.2)
- `llm` — LLM API client (implementation in v0.4)
- `mock` — mock LLM client for testing

### Tests
- 21 unit tests (all passing):
  - Rule matcher: 4 tests (CAS lookup, shape matching, specificity scoring)
  - Session flow: 13 tests (full flows, error handling, serialisation, Japanese)
  - JP tariff table: 4 tests

---

## Roadmap

| Version | Target | Description |
|---------|--------|-------------|
| 0.1.0 | ✅ 2026-05 | Core rule engine + Akinator session + JP tariff codes |
| 0.2.0 | ✅ 2026-05 | PubChem API integration (CAS / IUPAC name / SMILES / InChI lookup) |
| 0.3.0 | ✅ 2026-05 | SMILES functional-group detection (20 groups) + pipeline Priority 3 |
| 0.4.0 | ✅ 2026-05 | LLM trait hook + PromptBuilder (EN/JA) + MockLlmClassifier |
