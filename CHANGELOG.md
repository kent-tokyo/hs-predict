# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.5.0] — 2026-05-25

### Added

#### Mixture GRI classification (`src/mixture.rs`)

- New **`mixture`** module implementing WCO General Rules for Interpretation:
  - **Step 0** — Intended-use fast-path: Pharmaceutical → Ch.30,
    Agricultural pesticide formulations → Ch.38.08, Cosmetics → Ch.33, Food → Ch.21.
  - **GRI 3a** — All components in the same chapter → most specific heading by confidence.
  - **GRI 3b** — Dominant component (>50 % w/w) → essential character classification.
  - **GRI 3c** — Fallback: last heading numerically; sets `gray_zone` and
    `recommended_action = PriorConsultation`.
- `HsPipeline::classify()` now routes mixture products (`is_mixture() == true`) through
  the mixture classifier automatically (**Priority 0**, before all existing priorities).
- Closure-based design avoids circular crate dependency: `classify_mixture` accepts
  `classify_component: impl Fn(&ProductDescription) -> Result<HsPrediction>`.

#### Compliance risk flags (`src/types.rs`)

- **`GrayZone`** enum — identifies high-risk classification boundaries:
  - `Chapter29vs38` — organic compound vs. prepared formulation (most common misclassification)
  - `Chapter28vs29` — inorganic vs. organic (organometallic edge case)
  - `MixtureEssentialCharacterUnclear` — GRI 3c applied; no dominant component
- **`HsPrediction::gray_zone: Option<GrayZone>`** — `Some` when a boundary risk is detected.
- **`RecommendedAction::PriorConsultation`** — new variant recommending a formal advance
  ruling (事前教示) from customs authorities. Applied when a `GrayZone` boundary is
  detected or when GRI 3c is used.

#### Chapter 38 rules (`src/rules/chapter38.rs`)

- New `chapter38` module with `classify_by_intended_use()` and `special_chapter_by_use()`
  for use-case-driven mixture classification.
- Constant `CHAPTER38_CATCH_ALL_CODE` (`"382499"`) as the GRI 3c last-resort fallback.

#### Static rule table expansion (`src/rules/static_table.rs`)

- Expanded from **98 → 148 entries / 133 distinct compounds** (WCO-validated codes):
  - **Chapter 28** additions: H₂, N₂, O₂, CO₂, SO₂, NH₄Cl, MgCl₂, AlCl₃, FeCl₂,
    NiCl₂, CuCl₂, NaF, Al(OH)₃, K₂SO₄, Na₂S₂O₃, MgSO₄, NaNO₂, Ca(NO₃)₂, KNO₃,
    NaNO₃, Na₂Cr₂O₇, K₂Cr₂O₇, KMnO₄, sodium metabisulphite, and more.
  - **Chapter 29** additions: styrene, 1,3-butadiene, vinyl chloride, MEK, n-BuOAc,
    maleic anhydride, terephthalic acid, adipic acid, propylene oxide, glycerol, DEG,
    cyclohexanone, acrylic acid, MMA, acrylonitrile, ethylenediamine, HMDA,
    neopentyl glycol, TMP, pentaerythritol, TDI, MDI, n-butyl acrylate, methyl acrylate,
    propionic acid, butyric acid, succinic acid.
  - **Chapter 38** addition: activated carbon (380210).
- Corresponding Japan 統計品目番号 added to `jp_table.rs` (2026 tariff schedule).

#### Batch classification (`src/pipeline.rs`)

- `HsPipeline::classify_batch(&[ProductDescription]) -> Vec<Result<HsPrediction>>`
  — synchronous batch for multi-product workflows.
- `HsPipeline::classify_batch_with_llm(&[ProductDescription]) -> Vec<Result<HsPrediction>>`
  — async concurrent batch with LLM fallback (`llm` feature).

### Changed

- `HsPipeline::classify()` dispatches mixtures to the GRI classifier before all other priorities.
- `recommended_action_with_gz()` replaces the old `recommended_action()`: gray-zone
  presence upgrades `VerifyWithLlm` → `PriorConsultation`.
- `detect_gray_zone()` unified: previously two separate helpers (SMILES / static-rule)
  merged into one function accepting `organic_class: Option<&OrganicInorganic>`.
- `HsPipeline::with_mapping()` now validates that the supplied HS code is exactly
  6 ASCII digits, returning `HsPredictError::ValidationFailed` otherwise.
- `build_prediction()` (9-argument function) replaced by **`PredictionBuilder`** struct
  for named-field ergonomics; JP tariff lookup centralised inside `build()`.

### Fixed

#### Wrong HS codes in static rule table (bug-check audit)

Six codes were incorrect and have been corrected:

| CAS | Substance | Wrong | Correct | Reason |
|---|---|---|---|---|
| 7757-79-1 | Potassium nitrate (KNO₃) | `283410` (nitrites!) | `283421` | 2834.10 = nitrites; 2834.21 = potassium nitrates |
| 7631-99-4 | Sodium nitrate (NaNO₃) | `283421` (potassium!) | `283429` | 2834.21 is only "of potassium" |
| 7722-64-7 | Potassium permanganate | `284130` (Na₂Cr₂O₇!) | `284161` | 2841.30 = sodium dichromate |
| 10588-01-9 | Sodium dichromate | `281921` (non-existent!) | `284130` | 28.19 has only .10/.90 |
| 7778-50-9 | Potassium dichromate | `281929` (non-existent!) | `284150` | Heading 28.41 |
| 7664-41-7 / 7697-37-2 | Ammonia / Nitric acid | `281400` / `280800` (4-digit!) | `281410` / `280890` | 6-digit invariant violated |

- Ethanol `heading_description` typo `"(methanol)"` → `"(ethyl alcohol)"`.
- `jp_table.rs`: removed non-existent `281921`/`281929` entries; corrected
  dichromate and permanganate Japanese descriptions.

#### SMILES detector — aldehyde false-positive as alcohol (`src/smiles/detector.rs`)

- `smiles.ends_with("O")` was matching acetaldehyde (`CC=O`) as both Aldehyde and Alcohol.
  Fixed by checking `has_aldehyde` before the alcohol detection block.

#### Security hardening

- **`src/types.rs`** — `chapter()` and `heading()` used byte-slicing (`&s[..2]`/`&s[..4]`)
  which panics on multi-byte UTF-8 boundaries. Replaced with `.get(..n).unwrap_or(s)`.
- **`src/types.rs`** — `display_name()` SMILES short preview used `&s[..20]` (byte-slice).
  Replaced with `s.chars().take(20).collect::<String>()`.
- **`src/smiles/mod.rs`** — Added `MAX_SMILES_LEN = 4096` guard; inputs exceeding this
  return `None` immediately, preventing DoS via O(n²) pattern matching on large strings.
- **`src/session/mod.rs`** — `start()` called `.expect()` on a fallible operation in a
  public API. Replaced with `unwrap_or_else` returning a safe fallback question.
- **`src/llm/prompt.rs`** — Added `sanitize_context()`: strips ASCII control characters
  and caps `additional_context` at 500 chars to prevent prompt injection.
- **`src/pubchem/mod.rs`** — Added `PubChemClient::try_build() -> Result<Self>` to
  surface construction errors without panicking.
- **`src/mixture.rs`** — GRI 3a confidence sort used `partial_cmp().unwrap()` which
  panics on NaN. Replaced with `.unwrap_or(Ordering::Equal)`.

### Tests

- **120 unit tests** (up from 93), **14 doctests** — all pass; `cargo clippy -D warnings` clean.
- New regression tests:
  - `acetaldehyde_not_classified_as_alcohol` (smiles/detector)
  - `gri3a_nan_confidence_does_not_panic` (mixture)
  - `all_unknown_weights_falls_to_gri3c` (mixture)
  - `single_component_mixture_classifies_via_gri3a` (mixture)
  - `gri3b_exactly_50pct_is_not_dominant` (mixture)
- Fixed 4 pre-existing `clippy` warnings (`single_match`, `collapsible_if`,
  `format_in_format_args`) that were blocking `cargo clippy -D warnings`.

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
| 0.4.1 | ✅ 2026-05 | WASM companion crate (`hs-predict-wasm`) + Serialize additions |
| 0.5.0 | ✅ 2026-05 | Mixture GRI 3a/3b/3c · GrayZone · PriorConsultation · 133 compounds · batch · security |
| 0.5.1 | 📋 planned | npm publish · GitHub Actions CI · WASM tests |
