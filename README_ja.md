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
- **ハイブリッドパイプライン** — 混合物GRI → 静的ルールテーブル → SMILES官能基エンジン → LLMフォールバック（優先順位順）
- **物理的形状を考慮したマッチング** — 同一物質でも形状が違えば別HSコード（例: NaOH固体→2815.11、溶液→2815.12）
- **静的ルールテーブル 148エントリ / 133化合物** — 第28・29・38・72〜81類の主要工業化学品
- **SMILES官能基検出** *(v0.3)* — 20官能基、有機/無機分類、ヘディングレベルのヒント（信頼度 ≤ 0.70）
- **SMILES構造特徴エンジン** *(v0.5.1)* — 炭素数・水酸基数・環構造・C=C結合を解析し、ケトン・アルコール・カルボン酸・アルデヒドを**6桁HSサブヘディング**まで解決（信頼度最大0.90）
- **炭化水素・塩素化炭化水素エンジン** *(v0.5.2)* — 純炭化水素検出（ヘテロ原子なし）とC=C結合数カウントにより、HS 2901（非環式：エチレン・プロピレン・イソプレン等）、2902（環式：シクロヘキサン・ベンゼン・スチレン等）、2903（塩化メチレン・クロロホルム・四塩化炭素等）を6桁まで解決
- **混合物GRI分類** *(v0.5)* — GRI 3a（同一章）、GRI 3b（主成分 >50% w/w）、GRI 3c（数字上最後の見出し）を自動適用。医薬品（第30類）・化粧品（第33類）・食品（第21類）・農薬（第38.08）は用途別ルーティング
- **コンプライアンスリスクフラグ** *(v0.5)* — `GrayZone` で第28/29/38類の境界ケースを検出。`PriorConsultation` で事前教示照会を推奨
- **バッチ処理** *(v0.5)* — `classify_batch()` / `classify_batch_with_llm()` で複数品目を一括分類
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
 │  Priority 0: 混合物GRI分類器      (v0.5)                 │
 │  GRI 3a → 3b (>50% w/w) → 3c；用途別ルーティング        │
 │  （医薬品Ch.30/農薬Ch.38.08/化粧品Ch.33/食品Ch.21）     │
 └──────────────────────┬───────────────────────────────────┘
                        │ 混合物でない場合
                        ▼
 ┌──────────────────────────────────────────────────────────┐
 │  Priority 1: ユーザー提供マッピング  (信頼度 = 1.0)     │
 │  pipeline.with_mapping("64-19-7", "291511")              │
 └──────────────────────┬───────────────────────────────────┘
                        │ ミス
                        ▼
 ┌──────────────────────────────────────────────────────────┐
 │  Priority 2: 静的ルールテーブル    (133化合物)           │
 │  CAS + 形状 + 純度 → 6桁HSサブヘディング                 │
 └──────────────────────┬───────────────────────────────────┘
                        │ ミス
                        ▼
 ┌──────────────────────────────────────────────────────────┐
 │  Priority 3: SMILES構造特徴エンジン (v0.5.1)             │
 │  20官能基 + 構造特徴量                                   │
 │  → ケトン/酸/アルコール/アルデヒドは6桁サブヘディング   │
 │  → その他の官能基はヘディングヒント (信頼度 ≤ 0.70)     │
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
                gray_zone, recommended_action,
                jp_tariff_code, alternatives }
```

---

## 混合物GRI分類（v0.5）

`ProductDescription::mixture_components` を設定すると、WCO一般解釈規則（GRI）を自動適用します。

| ステップ | 規則 | 条件 |
|---|---|---|
| 0 | 用途別ルーティング | 医薬品→第30類 / 化粧品→第33類 / 食品調製品→第21類 / 農薬→第38.08 |
| 1 | GRI 3a | 全成分が同一章 → 最具体的見出しを採用 |
| 2 | GRI 3b | 主成分 > 50% w/w → その成分の分類を採用 |
| 3 | GRI 3b + LLM | 主成分なし → LLMに委譲（llm feature が必要） |
| 4 | GRI 3c | 数字上最後の見出し；信頼度0.40；`PriorConsultation` 推奨 |

```rust
use hs_predict::types::{ProductDescription, SubstanceIdentifier, MixtureComponent, IntendedUse};
use hs_predict::pipeline::HsPipeline;

let pipeline = HsPipeline::new();

let product = ProductDescription {
    identifier: SubstanceIdentifier::default(),
    physical_form: None,
    purity_pct: None,
    purity_type: None,
    intended_use: Some(IntendedUse::Agricultural),
    mixture_components: Some(vec![
        MixtureComponent {
            substance: SubstanceIdentifier::from_cas("1071-83-6"), // グリホサート
            weight_fraction_pct: Some(41.0),
            volume_fraction_pct: None,
            is_solvent: false,
        },
    ]),
    additional_context: None,
};

let p = pipeline.classify(&product)?;
// 農薬用途 → 3808.xx（第38類）
assert_eq!(p.chapter(), "38");
# Ok::<(), hs_predict::HsPredictError>(())
```

---

## コンプライアンスリスクフラグ（v0.5）

`HsPrediction` に `gray_zone` フィールドが追加され、境界ケースを検出できます。

```rust
use hs_predict::types::{GrayZone, RecommendedAction};

let p = pipeline.classify(&product)?;

match p.gray_zone {
    Some(GrayZone::Chapter29vs38) => {
        // 有機化合物の調製品 — 第29類 vs 第38類を要確認
    }
    Some(GrayZone::MixtureEssentialCharacterUnclear) => {
        // GRI 3c 適用 — 事前教示（binding tariff information）を強く推奨
    }
    _ => {}
}

if p.recommended_action == RecommendedAction::PriorConsultation {
    // 申告前に税関へ事前教示を照会する
}
```

| `GrayZone` バリアント | 意味 |
|---|---|
| `Chapter29vs38` | 有機化合物が用途・調製方法により第38類へシフトする可能性あり |
| `Chapter28vs29` | 有機金属境界 — 金属–炭素結合の有無が分類を決定する |
| `MixtureEssentialCharacterUnclear` | GRI 3c 適用（主成分なし）；事前教示を推奨 |

> **事前教示（先例教示）について**: グレーゾーンに該当する場合、最長5年遡及の追徴課税リスクを
> 回避するため、税関へのBinding Tariff Information（事前教示）申請を強く推奨します。

---

## バッチ処理（v0.5）

```rust
let products: Vec<ProductDescription> = vec![/* ... */];

// 同期バッチ（Priority 0〜3）
let results: Vec<Result<HsPrediction>> = pipeline.classify_batch(&products);

// LLMフォールバック付き非同期バッチ（Priority 4）
# #[cfg(feature = "llm")]
let results = pipeline.classify_batch_with_llm(&products).await;
```

---

## SMILES構造特徴エンジン（v0.5.1）

SMILESが利用可能な場合（ユーザー入力またはPubChem補完）、まず構造特徴量（炭素数・水酸基数・環構造・C=C結合）を解析し、主要化合物クラスを**6桁HSサブヘディング**まで解決します。

### 6桁サブヘディング解決

| 化合物クラス | 例 | SMILES | HSサブヘディング | 信頼度 |
|---|---|---|---|---|
| アセトン（3C ケトン） | アセトン | `CC(C)=O` | **291411** | 0.87 |
| MEK（4C ケトン） | メチルエチルケトン | `CCC(C)=O` | **291412** | 0.83 |
| MIBK（6C 非環式ケトン） | MIBK | `CC(=O)CC(C)C` | **291413** | 0.80 |
| シクロヘキサノン | シクロヘキサノン | `O=C1CCCCC1` | **291422** | 0.85 |
| アセトフェノン | アセトフェノン | `CC(=O)c1ccccc1` | **291431** | 0.82 |
| エタノール（2C 一価） | エタノール | `CCO` | **220710** | 0.85 |
| エチレングリコール（2C 二価） | エチレングリコール | `OCCO` | **290531** | 0.85 |
| グリセロール（3C 三価） | グリセロール | `OCC(O)CO` | **290541** | 0.85 |
| 酢酸（2C） | 酢酸 | `CC(=O)O` | **291521** | 0.90 |
| プロピオン酸（3C） | プロピオン酸 | `CCC(=O)O` | **291550** | 0.83 |
| アクリル酸（C=C） | アクリル酸 | `OC(=O)C=C` | **291611** | 0.87 |
| メタクリル酸 | メタクリル酸 | `CC(=C)C(=O)O` | **291613** | 0.87 |
| 安息香酸（芳香族） | 安息香酸 | `OC(=O)c1ccccc1` | **291631** | 0.85 |
| ベンズアルデヒド（芳香族） | ベンズアルデヒド | `O=Cc1ccccc1` | **291211** | 0.83 |
| アセトアルデヒド（2C） | アセトアルデヒド | `CC=O` | **291212** | 0.83 |

> **注意:** エタノールは**第22類**（変性していないエチルアルコール、容量80%以上）にルーティングされます。WCO分類実務に準拠した正しい挙動です。

### ヘディングレベルヒント（その他の官能基）

| 官能基 | HSヘディングヒント | 信頼度 |
|---|---|---|
| 酸無水物 | 29.15 | 0.65 |
| イソシアネート | 29.29 | 0.70 |
| ニトリル | 29.26 | 0.70 |
| エポキシド | 29.10 | 0.70 |
| スルホン酸 | 29.04 | 0.68 |
| アミド | 29.24 | 0.67 |
| エステル | 29.15 | 0.55 |
| フェノール | 29.07 | 0.67 |
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
assert_eq!(r.heading_hint.heading, Some(2914));
assert_eq!(r.heading_hint.subheading.as_deref(), Some("291411")); // 6桁まで解決！
assert!(r.heading_hint.confidence >= 0.85);

let r = classify_smiles("CCO").unwrap(); // エタノール → 第22類
assert_eq!(r.heading_hint.subheading.as_deref(), Some("220710"));
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
| CAS番号 | `1310-73-2` | yes |
| IUPAC系統名 | `sodium hydroxide` | yes（フォールバック） |
| SMILES | `[Na+].[OH-]` | yes |
| InChI | `InChI=1S/Na.H2O/h;1H/q+1;/p-1` | yes |
| InChIKey | `HEMHJVSKTPXQMS-UHFFFAOYSA-M` | yes |

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
hs-predict = { version = "0.5.2", features = ["pubchem"] }
```

---

## 静的ルールテーブルの例（133化合物 / 148エントリ）

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

全148エントリ（133化合物）は [`src/rules/static_table.rs`](src/rules/static_table.rs) を参照。

---

## ロードマップ

| バージョン | ステータス | 内容 |
|---|---|---|
| 0.1.0 | Released | コアルールエンジン + Akinatorセッション + 統計品目番号 |
| 0.2.0 | Released | PubChem API連携 |
| 0.3.0 | Released | SMILES官能基検出（20種） + Pipeline Priority 3 |
| 0.4.0 | Released | LLM trait hook + PromptBuilder(EN/JA) + MockLlmClassifier + WASM対応 |
| 0.4.1 | Released | WAMSコンパニオンクレート + Serialize追加 |
| 0.5.0 | Released | 混合物GRI分類 · GrayZone · PriorConsultation · 133化合物 · バッチ処理 · セキュリティ強化 |
| 0.5.1 | Released | SMILES構造特徴エンジン — ケトン・アルコール・カルボン酸・アルデヒドを6桁まで解決（信頼度最大0.90） |
| 0.5.2 | Released | 炭化水素エンジン（HS 2901/2902）・塩素化炭化水素エンジン（HS 2903）— イソプレン・シクロヘキサン・DCMなど13化合物追加 |
| 0.6.0 | Planned | npm公開 · GitHub Actions CI · WASMテスト |

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
