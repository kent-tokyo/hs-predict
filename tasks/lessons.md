# hs-predict 開発ログ — 教訓・判断記録

最終更新: 2026-05-24

---

## LLM設計: Trait Hook vs Built-in Client（v0.4）

**決定**: HTTPクライアントをライブラリに持たない。`LlmClassifier` trait をユーザーが実装する。

**理由**:
- HSコードの誤分類は通関申告上の法的リスクを伴う。LLMの非決定性をライブラリ内に持ち込むと、同一品物で毎回異なるコードが返るリスクがある
- プロバイダーロックイン（Anthropic/OpenAI）を避けたい
- APIキー管理をライブラリに持たせたくない（`Cargo.toml` に秘密情報が入る問題）
- PubChemは「事実データの補完」（決定論的）なので保持。LLMは「分類判断」（非決定論的）なので外出し

**実装**: `BoxFuture` + `Send + Sync` でオブジェクト安全な非同期trait。`async_trait` クレート不要。

---

## PubChem canonical SMILES のパターン注意点（v0.3）

SMILES のパターンマッチングでは、PubChemが返すcanonical SMILESの書き方に合わせる必要がある。

| 化合物 | 誤ったパターン | 正しいパターン |
|---|---|---|
| エチレンオキシド（エポキシド） | `C1OC1` | `C1CO1`（C-C-O環順） |
| フタル酸無水物（環状無水物） | `C(=O)OC(=O)` のみ | `O=C{n}OC(=O)` も必要 |
| ニトロベンゼン（ニトロ基） | `[N+](=O)[O-]` | `O=[N+]([O-])`（O=が先） |

**教訓**: PubChemの実際のSMILESを `pubchem.ncbi.nlm.nih.gov/compound/acetaldehyde` 等で確認してからパターンを書く。

---

## docs.rs ビルド失敗の原因（v0.2回帰）

`src/llm/` ディレクトリが存在するが `mod.rs` がない状態で `--all-features` ビルドすると
docs.rs がコンパイルエラーになる。空のディレクトリはモジュールとして認識されない。

**対策**: feature-gated モジュールのディレクトリは必ずスタブ `mod.rs` を置く。

---

## wasm-bindgen をネイティブでテストする際の注意（v0.4 WASM）

`wasm-bindgen` の `JsValue`/`JsString` 操作はJSランタイムが必要。
ネイティブ（`cargo test`）で `#[test]` から直接呼ぶと `SIGABRT` でクラッシュする。

**対策**: wasm-bindgen を使うテストは `#[cfg(all(test, target_arch = "wasm32"))]` でゲートし、
`wasm-bindgen-test` / `wasm-pack test --node` で実行する。
コア分類ロジックは `hs-predict` 側のテストで網羅すれば十分。

---

## `HeadingHint` に `&'static str` フィールドがある問題（v0.4 WASM）

`HeadingHint::rationale: &'static str` は `serde::Deserialize` を derive できない
（デシリアライザーのライフタイムを借用できないため）。

**対策**: `Serialize` のみを derive する（WASMでは JS に送るだけなので Deserialize は不要）。
将来 rationale を `String` に変えれば両方 derive できる（breaking change）。

---

## workspace の `[profile.release]` はルートに置く

`Cargo.toml` が workspace ルートになった場合、サブクレートの `[profile.release]` は無視される。
`wasm-pack build` のリリース最適化（`opt-level = "z"`, `lto = true`, `panic = "abort"`）を
有効にするには、workspace ルートの `Cargo.toml` に書く必要がある。

---

## crates.io トークンの有効期限

`cargo publish` 用のトークンは使用期限がある（またはセッション終了で無効化）。
公開前に必ず `https://crates.io/settings/tokens` で新しいトークンを発行する。

---

## `HsPipeline` の `Debug` 実装

`Arc<dyn LlmClassifier>` は `Debug` を実装していないため、`#[derive(Debug)]` が使えない。
手動で `fmt::Debug` を実装し、LLMフィールドは `"<dyn LlmClassifier>"` という文字列で表示する。

---

## Cargo workspace + crates.io publish

workspace 化後も `cargo publish` は個別クレートごとに行う（`-p hs-predict` など）。
workspace root の Cargo.toml に `[package]` があれば `cargo publish` でそのクレートを公開できる。
`hs-predict-wasm` は別途 `cargo publish -p hs-predict-wasm` で公開する必要がある。
