# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
| 0.4.0 | 2027-Q1 | LLM API integration (Claude) with context-aware prompting |
