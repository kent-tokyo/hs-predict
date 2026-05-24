# hs-predict タスクリスト

最終更新: 2026-05-24

## プロジェクト概要

Rustライブラリ `hs-predict` — 化学品輸出入実務のためのHSコード（国際共通6桁）予測ライブラリ。

### アーキテクチャ決定

| 項目 | 決定 |
|---|---|
| 予測方式 | ハイブリッド（ルールベース優先 → LLM フォールバック） |
| **UXコンセプト** | **Akinatorスタイルの段階的対話**（最重要） |
| 識別子入力 | CAS番号 / **IUPAC名のみ**（別名・商品名は不可） / SMILES / InChI |
| IUPAC名→SMILES | `chem-name-resolver` クレートで自動変換 |
| データ補完 | PubChem API — v0.2（CAS/IUPAC名/SMILES/InChI検索） |
| HS体系 | 国際共通6桁（HS2022） |
| 形状対応 | 固体/溶液/気体/粉末等で同一物質でもコードが異なる |
| 混合物対応 | 各成分の重量割合を段階的に入力 |
| LLM設計 | **Trait hookパターン** — ライブラリはHTTPクライアントを持たない |
| WASM | workspace サブクレート `hs-predict-wasm`（wasm-bindgen 0.2.121） |
| MSRV | 1.75（native async fn in traits） |
| 公開先 | crates.io |
| GitHub | https://github.com/kent-tokyo/hs-predict |

### Akinatorスタイル質問フロー

```
Q1: CAS / IUPAC名 / SMILES / InChIKey を入力
    └─ PubChem（v0.2）ヒット → 確認して完了（最短1問）
    └─ 未発見 → Q2へ

Q2: 混合物ですか？ [Yes / No]
    ├─ Yes → 成分数 → 各成分のCAS/IUPAC名 → 各成分の重量割合(w/w%)
    └─ No  → 物理的形状? [固体/粉末/液体/溶液/気体/…]
                └─ 溶液 → 濃度(w/w%)
                └─ 用途? [工業/医薬品/農薬/食品/化粧品]
                └─ 有機化合物? → 官能基は?（複数選択）
                → HS code確定（最大7問）
```

---

## 凡例

- `[x]` 実装済み・テスト通過
- `[~]` 部分実装（プレースホルダー）
- `[ ]` 未着手

---

## Phase 1: プロジェクト基盤 ✅

- [x] `cargo init --lib` でプロジェクト作成
- [x] `Cargo.toml` 設定
  - [x] パッケージメタデータ（name/version/edition/rust-version=1.75）
  - [x] キーワード（"hs-code","customs","chemical","trade","llm"）
  - [x] カテゴリ（"science","api-bindings"）
  - [x] ライセンス（MIT OR Apache-2.0）
  - [x] feature flags（default=[]/pubchem/llm/mock）
  - [x] 依存クレート（thiserror/serde/serde_json/chem-name-resolver/futures 常時 + reqwest/moka/governor/tokio optional）
  - [x] workspace 化（hs-predict-wasm をメンバー追加）
  - [x] GitHub リポジトリ URL（https://github.com/kent-tokyo/hs-predict）
- [x] ディレクトリ構造作成
- [x] `CHANGELOG.md` 作成（Keep a Changelog形式）
- [x] `.gitignore` 設定・git管理下に追加
- [ ] GitHub Actions CI 設定
  - [ ] `cargo test --all-features`
  - [ ] `cargo clippy --all-features -- -D warnings`
  - [ ] `cargo fmt --check`
  - [ ] MSRV 1.75 での `cargo build` 確認
- [ ] LICENSE-MIT / LICENSE-APACHE ファイル作成

---

## Phase 2: コア型定義 ✅

- [x] `src/types.rs` — 入出力の基本型
  - [x] `SubstanceIdentifier`（cas/smiles/iupac_name/inchi/inchi_key/cid）
  - [x] `PhysicalForm` enum（Solid/Powder{粒径}/Granules/Liquid/Solution{溶媒,濃度}/Gas/Foil{厚さ}/Ingot/Unknown）
  - [x] `PurityType` / `MixtureComponent` / `ProductDescription` / `IntendedUse`
  - [x] `HsPrediction`（hs_code/heading_description/confidence/source/notes/alternatives/recommended_action/jp_tariff_code/jp_tariff_year）
  - [x] `PredictionSource` / `RecommendedAction` / `OrganicInorganic`（Serialize対応）
- [x] `src/error.rs` — `HsPredictError`（thiserror）
  - [x] 全エラーバリアント実装済み（入力/セッション/パイプライン/LLM/PubChem）

---

## Phase 3: Akinatorスタイル対話エンジン ✅

- [x] `src/session/question.rs`
  - [x] `Question` / `Answer` / `QAPair` / `SessionResult`（Serialize対応）
- [x] `src/session/state.rs` — `ClassificationState`
- [x] `src/session/flow.rs` — 質問決定木（7ステップ分岐）
- [x] `src/session/mod.rs` — `ClassificationSession`
  - [x] `new()` / `new_ja()` / `start()` / `answer()` / `to_product_description()`
  - [x] IUPAC名→SMILES自動変換（chem-name-resolver）
  - [x] Serialize/Deserialize（セッション一時停止・再開）
  - [x] ユニットテスト 13件（セッションフロー/シリアライズ/日本語）

---

## Phase 4: 静的HSルールテーブル ✅

- [x] `src/rules/matcher.rs` — `find_best_rule(cas, form, purity)`（具体性スコア）
- [x] `src/rules/static_table.rs` — 98品目（第28・29・72〜81類）
- [x] `src/rules/jp_table.rs` — 統計品目番号（9桁）約70件（実行関税率表2026-04-01）

---

## Phase 5: SMILES解析エンジン（v0.3）✅

- [x] `src/smiles/detector.rs`
  - [x] `FunctionalGroup` enum（20種類）+ Serialize/Deserialize
  - [x] `classify_organic(smiles)` — 有機/無機/有機金属判別
  - [x] `detect_functional_groups(smiles)` — 優先順位付きパターンマッチング
- [x] `src/smiles/chapter_map.rs`
  - [x] `HeadingHint` + Serialize
  - [x] `map_to_heading(groups, organic_class)` — 20エントリ優先マップ
- [x] `src/smiles/mod.rs`
  - [x] `SmilesClassification` + Serialize
  - [x] `classify_smiles(smiles)` — パブリックエントリポイント
- [x] Pipeline Priority 3 統合（SMILES→4桁ヘディング hint）
- [x] ユニットテスト 40件（detector 19件 + chapter_map 11件 + 統合 10件）

---

## Phase 6: PubChem連携（v0.2）✅

- [x] `src/pubchem/client.rs` — `PubChemClient`
  - [x] CAS/InChIKey/InChI/SMILES/IUPAC名 → PubChemCompound
  - [x] TTLキャッシュ（moka: 1000件, 24時間TTL）
  - [x] レートリミッター（governor: 5req/s）
  - [x] `PubChemClientBuilder`（テスト用カスタムベースURL対応）
- [x] `HsPipeline::with_pubchem()` / `enrich()` 統合

---

## Phase 7: 混合物分類エンジン

- [ ] `src/mixture.rs` — `MixtureClassifier`
  - [ ] 特殊分類チェック（医薬品Ch.30/農薬Ch.38/化粧品Ch.33/食品添加物Ch.21）
  - [ ] GRI 3a（同一章→最具体的見出し）
  - [ ] GRI 3b → LLM委譲（本質的特性判定）
  - [ ] GRI 3c（数字上最後の見出し、低信頼度0.40）
  - [ ] 主成分優先ロジック（>50% w/w）

---

## Phase 8: メインパイプライン ✅

- [x] `src/pipeline.rs` — `HsPipeline`
  - [x] `PipelineConfig`（thresholds）
  - [x] `classify(product)` — Priority 1-3 同期分類
  - [x] `classify_with_llm(product)` — Priority 4 非同期分類（`llm` feature）
  - [x] `with_mapping()` / `with_config()` / `with_llm()` / `with_pubchem()`
  - [x] 手動 `Debug` impl（`Arc<dyn LlmClassifier>` 非Debug対応）
  - [ ] `classify_batch()` — 並列バッチ処理

---

## Phase 9: LLM統合（v0.4）✅ — Trait Hook方式

- [x] `src/llm/mod.rs` — `LlmClassifier` trait（BoxFuture、Send+Sync）
  - [x] `LlmPrompt`（system_text/user_text/smiles_analysis）
  - [x] `LlmResponse` / `LlmAlternative`（Serialize/Deserialize）
  - [x] `parse_llm_json()` — マークダウンフェンス除去 + JSON解析
- [x] `src/llm/prompt.rs` — `PromptBuilder`（EN/JA両対応）
  - [x] システムプロンプト（HS専門家ロール/JSON出力仕様/信頼度ガイド）
  - [x] ユーザーメッセージ（識別子/形状/純度/混合成分/SMILESヒント）
- [x] `src/llm/mock.rs` — `MockLlmClassifier`（`mock` feature）
  - [x] SMILES解析から決定論的にHS導出
  - [x] カスタムデフォルトコード設定可能
- [x] Pipeline Priority 4 統合（hs_code 6桁バリデーション/章整合性チェック）
- [x] ユニットテスト 19件（parse 5件 + prompt 9件 + mock 5件）
- [x] Pipeline統合テスト 5件（mock feature）

**設計方針**: HTTPクライアントはライブラリに持たない。ユーザーが任意のクライアントで
`LlmClassifier` を実装し、ライブラリは `LlmPrompt`・バリデーション・`parse_llm_json` を提供。

---

## Phase 10: WASM対応（v0.4）✅

- [x] workspace 化（root Cargo.toml に `[workspace]` 追加）
- [x] `hs-predict-wasm/Cargo.toml`（cdylib+rlib, wasm-bindgen 0.2.121）
- [x] `hs-predict-wasm/src/lib.rs`
  - [x] `classify_smiles(smiles: &str) -> JsValue` — SMILES→官能基+ヘディング
  - [x] `classify_product(json: &str) -> Result<JsValue, JsValue>` — ルールエンジン全体
  - [x] `WasmSession` struct（`new()` / `new_ja()` / `start()` / `answer()` / `classify()`）
  - [x] `serde-wasm-bindgen` + `to_js<T: Serialize>` ヘルパー
- [x] WASM ビルド確認（`wasm32-unknown-unknown`）
- [x] `wasm-pack build --target web --release` → `pkg/` 317KB
- [x] 既存テスト全パス（93 unit + 13 doc）
- [ ] npm パッケージとして公開（`@kent-tokyo/hs-predict-wasm`）
- [ ] CDN用 ESM バンドル（WASM base64インライン、build_cdn.mjs）
- [ ] `wasm-pack test --node` でWASMテスト実行

---

## Phase 11: テスト・ドキュメント

- [x] ユニットテスト 93件（全パス）
- [x] Doctestサンプル 13件（全パス）
- [x] `README.md` — 英語版（v0.4対応済み）
- [x] `README_ja.md` — 日本語版（v0.4対応済み）
- [x] `CHANGELOG.md`（v0.1〜v0.4）
- [ ] 統合テスト
  - [ ] `tests/session_flow.rs` — 主要フロー
  - [ ] `tests/rule_engine.rs` — 主要~100品のHS正解率チェック
  - [ ] `tests/pubchem_mock.rs` — wiremockでHTTPモック
- [ ] GitHub Actions CI

---

## Phase 12: crates.io 公開

- [x] v0.1.0 公開済み（2026-05-24）
- [x] v0.2.0 公開済み（2026-05-24）
- [x] v0.3.0 公開済み（2026-05-24）
- [x] v0.4.0 公開済み（2026-05-24）
- [ ] v0.5.0 — hs-predict-wasm npm公開 / 混合物GRI分類 / GitHub Actions CI

---

## ロードマップ

| バージョン | ステータス | 内容 |
|---|---|---|
| v0.1.0 | ✅ 2026-05 | コアルールエンジン + Akinatorセッション + 統計品目番号 |
| v0.2.0 | ✅ 2026-05 | PubChem API連携 |
| v0.3.0 | ✅ 2026-05 | SMILES官能基検出（20種） + Pipeline Priority 3 |
| v0.4.0 | ✅ 2026-05 | LLM trait hook + PromptBuilder(EN/JA) + MockLlmClassifier + WASM対応 |
| v0.5.0 | 🔜 | 混合物GRI分類 / npm公開 / GitHub Actions CI |
