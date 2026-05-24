# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
| 0.2.0 | 2026-Q3 | PubChem API integration (CAS / IUPAC name / SMILES / InChI lookup) |
| 0.3.0 | 2026-Q4 | SMILES functional-group detection (20 groups) + mixture GRI classification |
| 0.4.0 | 2027-Q1 | LLM API integration (Claude) with context-aware prompting |
