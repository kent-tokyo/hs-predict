# hs-predict

[![Crates.io](https://img.shields.io/crates/v/hs-predict.svg)](https://crates.io/crates/hs-predict)
[![docs.rs](https://docs.rs/hs-predict/badge.svg)](https://docs.rs/hs-predict)
[![CI](https://github.com/kent-tokyo/hs-predict/actions/workflows/ci.yml/badge.svg)](https://github.com/kent-tokyo/hs-predict/actions)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#ライセンス)

**化学品のHSコード（国際統一商品分類）予測ライブラリ。**

`hs-predict` は **Akinatorスタイルの段階的対話** を採用しています。分類に必要な情報だけを1問ずつ問いかけ、ルールベースエンジンとLLMを組み合わせた6桁のHS2022コードを予測します。

> **免責事項**: 本ライブラリの予測結果はあくまで参考情報です。税関申告の唯一の根拠として使用しないでください。必ず貿易コンプライアンスの専門家または関税当局に確認してください。

---

## 特徴

- **Akinatorスタイルのユーザー体験** — HSコード絞り込みに必要な質問だけを行い、最初から全情報を入力させない
- **ハイブリッドパイプライン** — 静的ルールテーブル → SMILES官能基エンジン → LLMフォールバック（優先順位順）
- **物理的形状を考慮したマッチング** — 同一物質でも形状が違えば別HSコード（例: NaOH固体→2815.11、溶液→2815.12）
- **静的ルールテーブル 98品目** — 第28・29・72〜81類の主要工業化学品
- **SMILES官能基検出** *(v0.3)* — 20官能基、有機/無機分類、ヘディングレベルのヒント（信頼度 ≤ 0.70）
- **混合物対応** — 各成分の識別子と重量割合（w/w%）を順番に入力
- **IUPAC名→SMILES自動変換** — [`chem-name-resolver`](https://crates.io/crates/chem-name-resolver) を使用
- **PubChem連携** *(v0.2、`pubchem` フィーチャー)* — CAS/IUPAC/SMILESからの識別子自動補完（決定論的）
- **LLM連携** *(v0.4、`llm` フィーチャー)* — **Trait hook設計**: 任意のLLMクライアントを実装して接続。ライブラリはプロンプト構築・レスポンス検証を担う
- **WebAssembly対応** *(v0.4)* — `hs-predict-wasm` サブクレートでブラウザから直接呼び出し可能
- **統計品目番号** — 実行関税率表2026-04-01に基づく9桁コードを全結果に含む

---

## クイックスタート

### 対話モード（Akinatorスタイル）

```rust
use hs_predict::session::{ClassificationSession, Answer, SessionResult};
use hs_predict::pipeline::HsPipeline;

let mut session = ClassificationSession::new();
let pipeline = HsPipeline::new();

let q = session.start();
println!("{}", q.prompt());
// "CAS番号、IUPAC名、SMILES、InChIKey のいずれかを入力してください"

match session.answer(Answer::Text("1310-73-2".to_string()))? {
    SessionResult::NeedMoreInfo { next_question } => {
        println!("次の質問: {}", next_question.prompt()); // "混合物ですか？"
    }
    SessionResult::Ready => {
        let product = session.to_product_description();
        let prediction = pipeline.classify(&product)?;
        println!("HSコード: {}", prediction.display()); // "28.15.11"
        if let Some(jp) = &prediction.jp_tariff_code {
            println!("統計品目番号: {}", jp);            // "281511000"
        }
    }
    SessionResult::RequiresLlm => { /* llm feature を有効にしてください */ }
}
# Ok::<(), hs_predict::HsPredictError>(())
```

### 日本語セッション

```rust
use hs_predict::session::ClassificationSession;

let mut session = ClassificationSession::new_ja(); // 日本語プロンプト
let q = session.start();
println!("{}", q.prompt()); // "CAS番号、IUPAC名、SMILES、InChIKey のいずれかを入力してください"
```

### ダイレクトモード（CAS番号と形状がわかっている場合）

```rust
use hs_predict::pipeline::HsPipeline;
use hs_predict::types::{ProductDescription, SubstanceIdentifier, PhysicalForm};

let pipeline = HsPipeline::new();

let product = ProductDescription {
    identifier: SubstanceIdentifier::from_cas("1310-73-2"), // 水酸化ナトリウム
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

## 分類パイプライン

```
入力: ProductDescription
        │
        ▼
 ┌──────────────────────────────────────────────────────────┐
 │  Priority 1: ユーザー提供マッピング  (信頼度 = 1.0)     │
 │  pipeline.with_mapping("64-19-7", "291511")              │
 └──────────────────────┬───────────────────────────────────┘
                        │ ミス
                        ▼
 ┌──────────────────────────────────────────────────────────┐
 │  Priority 2: 静的ルールテーブル    (98品目)              │
 │  CAS + 形状 + 純度 → 6桁HSサブヘディング                 │
 └──────────────────────┬───────────────────────────────────┘
                        │ ミス
                        ▼
 ┌──────────────────────────────────────────────────────────┐
 │  Priority 3: SMILES官能基エンジン  (v0.3)                │
 │  20官能基 → ヘディングレベルヒント (信頼度 ≤ 0.70)      │
 └──────────────────────┬───────────────────────────────────┘
                        │ ミス / 低信頼度
                        ▼
 ┌──────────────────────────────────────────────────────────┐
 │  Priority 4: LLM分類器           (v0.4、trait hook)      │
 │  impl LlmClassifier for YourClient { ... }               │
 └──────────────────────┬───────────────────────────────────┘
                        │
                        ▼
                   HsPrediction
              { hs_code, confidence, notes,
                jp_tariff_code, recommended_action }
```

---

## SMILES官能基検出（v0.3）

SMILESが利用可能な場合（ユーザー入力またはPubChem補完）、以下の官能基を検出してHSヘディングのヒントを生成します：

| 官能基 | HSヘディングヒント | 信頼度 |
|---|---|---|
| 酸無水物 | 29.15 | 0.65 |
| イソシアネート | 29.29 | 0.70 |
| ニトリル | 29.26 | 0.70 |
| エポキシド | 29.10 | 0.70 |
| スルホン酸 | 29.04 | 0.68 |
| アミド | 29.24 | 0.67 |
| アルデヒド | 29.12 | 0.67 |
| ケトン | 29.14 | 0.67 |
| カルボン酸 | 29.15 | 0.60 |
| エステル | 29.15 | 0.55 |
| フェノール | 29.07 | 0.67 |
| アルコール | 29.05 | 0.60 |
| アミン | 29.21 | 0.63 |
| 有機ハロゲン化物 | 29.03 | 0.65 |
| エーテル | 29.09 | 0.63 |
| チオール / スルフィド | 29.30 | 0.65 |
| リン酸エステル | 29.20 | 0.62 |
| ニトロ | 29.04 | 0.60 |
| 無機化合物（C–C/C–H結合なし） | 第28類 | 0.55 |

```rust
use hs_predict::smiles::classify_smiles;

let r = classify_smiles("CC(C)=O").unwrap(); // アセトン
assert_eq!(r.heading_hint.heading, Some(2914)); // 29.14 ケトン
```

---

## LLM連携 — 設計方針（v0.4）

### なぜTrait hookなのか

HSコードの誤分類は通関申告上の法的・財務的リスクを伴います。LLM APIクライアントをライブラリに直接組み込むと：

- **特定プロバイダーへの依存**（Anthropic、OpenAI など）が生じる
- **コンプライアンス上の非決定性** — 同一品物が呼び出しごとに異なるコードを返すリスク
- **APIキー管理の負担** をライブラリに持ち込む（`Cargo.toml` に機密情報）
- **ネットワーク遅延・障害** を同期的な分類呼び出しに混入させる

代わりに `hs-predict` はTrait hookを定義します。任意のHTTPクライアント・モデル・プロンプトカスタマイズで実装し、ライブラリは構造化された入力と出力検証を提供します。

```rust
// v0.4 — 任意のLLMクライアントでこのtraitを実装
use hs_predict::llm::{LlmClassifier, LlmPrompt, LlmResponse};
use futures::future::BoxFuture;

struct MyClaudeClient { api_key: String }

impl LlmClassifier for MyClaudeClient {
    fn classify<'a>(&'a self, prompt: &'a LlmPrompt) -> BoxFuture<'a, hs_predict::Result<LlmResponse>> {
        Box::pin(async move {
            // 1. prompt.system_text / prompt.user_text を使ってLLM APIを呼び出す
            // 2. JSONレスポンスを LlmResponse に変換（parse_llm_json() ヘルパーが利用可能）
            // 3. ライブラリがhs_codeのフォーマット検証と章整合性チェックを行う
            todo!()
        })
    }
}

// パイプラインに接続 — APIキーはライブラリに保持しない
let pipeline = HsPipeline::new().with_llm(MyClaudeClient { api_key: "...".into() });
let prediction = pipeline.classify_with_llm(&product).await?;
```

ライブラリが提供するもの：
- `LlmPrompt` — 事前構築済みシステムプロンプト + ユーザーメッセージ（製品情報 + SMILESヒント）
- `LlmResponse` — 期待される返却型（`hs_code`、`confidence`、`rationale`、`alternatives`）
- `PromptBuilder` — EN/JA両対応のプロンプトビルダー
- 6桁コード検証と章整合性チェック
- `MockLlmClassifier`（`mock` フィーチャー）— テスト用スタブ

---

## WebAssembly対応（v0.4）

`hs-predict-wasm` サブクレートにより、分類エンジン全体をブラウザから呼び出せます。

```bash
cd hs-predict-wasm
wasm-pack build --target web --release
# → pkg/ に JS バインディング生成（約317KB）
```

```js
import init, { classify_smiles, classify_product, WasmSession }
  from './pkg/hs_predict_wasm.js';
await init();

// 1. SMILES直接分類
const r = classify_smiles('CC(O)=O');
// → { organic_class: "organic", heading_hint: { heading: 2915, confidence: 0.6, ... } }

// 2. ルールエンジン全体
const pred = classify_product(JSON.stringify({
  identifier: { cas: "1310-73-2" }, physical_form: "Solid",
  purity_pct: null, purity_type: null, mixture_components: null,
  intended_use: null, additional_context: null
}));
// → { hs_code: "281511", confidence: 1.0, ... }

// 3. Akinatorセッション
const session = new WasmSession();          // または WasmSession.new_ja()
const q = session.start();
// → { step: "Identifier", prompt: "CAS番号...", type: "text" }
const r = session.answer(JSON.stringify({ Text: "1310-73-2" }));
// → { type: "NeedMoreInfo", next_question: { ... } }
// Ready になったら:
const prediction = session.classify();
```

---

## PubChem連携（v0.2）

PubChem連携は不足している識別子フィールドを分類前に補完します。
**事実データの取得**（決定論的）であり、LLMフォールバックとは役割が異なります。

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

pipeline.enrich(&mut product).await?;  // SMILES、InChI、IUPAC名を補完
let prediction = pipeline.classify(&product)?;
println!("{}", prediction.display()); // "28.15.11"
# Ok(())
# }
```

---

## 対話フロー

```
Q1: CAS番号 / IUPAC名 / SMILES / InChIKey を入力してください
     │
     ├─ PubChem自動補完 (pubchem feature) ──────────────────────┐
     │                                                            │
     ▼                                                            ▼
Q2: 混合物ですか？
     │
     ├─ はい ──► Q: 成分は何種類？
     │               └─ 各成分について:
     │                   ├─ Q: 識別子は？
     │                   └─ Q: 重量割合(w/w%)は？
     │
     └─ いいえ ─► Q3: 物理的形状は？
                    （固体 / 粉末 / 顆粒 / 液体 /
                      溶液 / 気体 / 箔 / インゴット / 不明）
                     │
                     ├─ 溶液 ──► Q: 濃度(w/w%)は？
                     │
                     ▼
                Q4: 用途は？
                    （工業用 / 医薬品 / 農薬 /
                      食品 / 化粧品 / その他）
                     │
                     ├─ SMILESなし ──► Q5: 有機/無機/有機金属？
                     │                      │
                     │                      └─ 有機 ──► Q6: 官能基は？
                     ▼
                 分類パイプライン（Priority 1〜4）
```

---

## 対応識別子フォーマット

| フォーマット | 例 | 自動検出 |
|---|---|---|
| CAS番号 | `1310-73-2` | ✅ |
| IUPAC系統名 | `sodium hydroxide` | ✅（フォールバック） |
| SMILES | `[Na+].[OH-]` | ✅ |
| InChI | `InChI=1S/Na.H2O/h;1H/q+1;/p-1` | ✅ |
| InChIKey | `HEMHJVSKTPXQMS-UHFFFAOYSA-M` | ✅ |

> **注意**: テキスト入力として受け付けるのは **IUPAC系統名のみ** です。俗称・商品名（例：「苛性ソーダ」「カセイソーダ」）は対応していません。

---

## フィーチャーフラグ

| フラグ | 説明 | 追加される依存クレート |
|---|---|---|
| *(なし)* | ルールベース + SMILESエンジン（Priority 1–3） | — |
| `pubchem` | PubChem識別子補完 | `reqwest`, `moka`, `governor` |
| `llm` | `LlmClassifier` trait + Pipeline Priority 4 | — |
| `mock` | `MockLlmClassifier`（テスト用、llm を含意） | — |

```toml
[dependencies]
hs-predict = { version = "0.4", features = ["pubchem"] }
```

---

## 静的ルールテーブルの例（98品目）

| CAS番号 | 物質名 | 形状 | HS2022コード |
|---|---|---|---|
| 1310-73-2 | 水酸化ナトリウム | 固体 | 2815.11 |
| 1310-73-2 | 水酸化ナトリウム | 溶液 | 2815.12 |
| 7664-93-9 | 硫酸 | 全形状 | 2807.00 |
| 7697-37-2 | 硝酸 | ≥ 98% | 2808.10 |
| 7697-37-2 | 硝酸 | < 98% | 2808.90 |
| 7664-41-7 | アンモニア | 気体 | 2814.10 |
| 7664-41-7 | アンモニア | 溶液 | 2814.20 |
| 7429-90-5 | アルミニウム | インゴット（純度≥99%） | 7601.10 |
| 7429-90-5 | アルミニウム | 粉末 | 7603.10 |
| 7429-90-5 | アルミニウム | 箔 | 7607.11 |
| 67-56-1 | メタノール | 液体 | 2905.11 |
| 64-17-5 | エタノール | 液体 | 2207.10 |
| 67-64-1 | アセトン | 液体 | 2914.11 |

全98品目は [`src/rules/static_table.rs`](src/rules/static_table.rs) を参照。

---

## ロードマップ

| バージョン | ステータス | 内容 |
|---|---|---|
| 0.1.0 | ✅ リリース済み | コアルールエンジン + Akinatorセッション + 統計品目番号 |
| 0.2.0 | ✅ リリース済み | PubChem API連携 |
| 0.3.0 | ✅ リリース済み | SMILES官能基検出（20種） + Pipeline Priority 3 |
| 0.4.0 | ✅ リリース済み | LLM trait hook + PromptBuilder(EN/JA) + MockLlmClassifier + WASM対応 |

---

## サポートするRustバージョン（MSRV）

Rust **1.75** 以上。

---

## コントリビューション

バグ報告、静的ルールテーブルへの追加、PRを歓迎します。  
静的ルールテーブルへの新規エントリを追加する場合は、HS2022品目表の該当章・号注を引用してください。

---

## ライセンス

以下のいずれかのライセンスのもとで提供されます:

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

どちらを選んでもかまいません。
