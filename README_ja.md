# hs-predict

[![Crates.io](https://img.shields.io/crates/v/hs-predict.svg)](https://crates.io/crates/hs-predict)
[![docs.rs](https://docs.rs/hs-predict/badge.svg)](https://docs.rs/hs-predict)
[![CI](https://github.com/kent-tokyo/hs-predict/actions/workflows/ci.yml/badge.svg)](https://github.com/kent-tokyo/hs-predict/actions)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#ライセンス)

**化学品のHSコード（国際統一商品分類）予測ライブラリ。**

`hs-predict` は **Akinatorスタイルの段階的対話** を採用しています。分類に必要な情報だけを1問ずつ問いかけ、ルールベースエンジンとLLMを組み合わせて6桁のHS2022コードを推測します。

> **免責事項**: 本ライブラリの予測結果はあくまで参考情報です。税関申告の唯一の根拠として使用しないでください。必ず貿易コンプライアンスの専門家または関税当局に確認してください。

---

## 特徴

- **Akinatorスタイルのユーザー体験** — HS絞り込みに必要な質問だけを行い、最初から全情報を入力させない
- **ハイブリッドパイプライン** — 静的ルールテーブル → SMILESルールエンジン → LLMフォールバック（優先順位順）
- **物理的形状を考慮したマッチング** — 同一物質でも形状が違えば別HSコード（例: NaOH固体→2815.11、溶液→2815.12）
- **混合物対応** — 各成分の識別子と重量割合（w/w%）を順番に入力
- **IUPAC名→SMILES自動変換** — [`chem-name-resolver`](https://crates.io/crates/chem-name-resolver) を使用
- **PubChem連携** *(v0.2、`pubchem` フィーチャー)* — CAS/IUPAC/SMILESからの自動データ補完
- **LLM連携** *(v0.4、`llm` フィーチャー)* — Claude/OpenAI互換API、章整合性バリデーション付き

---

## クイックスタート

### 対話モード（Akinatorスタイル）

```rust
use hs_predict::session::{ClassificationSession, Answer, SessionResult};
use hs_predict::pipeline::HsPipeline;

let mut session = ClassificationSession::new();
let pipeline = HsPipeline::new();

// セッション開始 — 最初の質問を受け取る
let q = session.start();
println!("{}", q.prompt());
// "CAS番号、IUPAC名、SMILES、InChIKey のいずれかを入力してください"

// CAS番号で回答（フォーマットは自動検出）
match session.answer(Answer::Text("1310-73-2".to_string()))? {
    SessionResult::NeedMoreInfo { next_question } => {
        println!("次の質問: {}", next_question.prompt());
        // "混合物ですか？"
    }
    SessionResult::Ready => {
        let product = session.to_product_description();
        let prediction = pipeline.classify(&product)?;
        println!("HSコード: {}", prediction.display()); // "28.15.11"
        println!("信頼度: {:.0}%", prediction.confidence * 100.0);
    }
    SessionResult::RequiresLlm => {
        // `llm` フィーチャーを有効にしてAPIキーを設定してください
    }
}
# Ok::<(), hs_predict::HsPredictError>(())
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

## 対話フロー

```
Q1: CAS番号 / IUPAC名 / SMILES / InChIKey を入力してください
     │
     ├─ PubChem自動補完 (v0.2) ────────────────────────────────┐
     │                                                           │
     ▼                                                           ▼
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
                      食品 / 化粧品 / 研究用 / その他）
                     │
                     ├─ SMILESなし ──► Q5: 有機/無機/有機金属？
                     │                      │
                     │                      └─ 有機 ──► Q6: 官能基は？
                     ▼
                 ┌─────────────────────────────────────────────┐
                 │            ハイブリッドパイプライン          │
                 │  1. ユーザー提供マッピング  (信頼度 = 1.0)  │
                 │  2. 静的ルールテーブル（約50品目）          │
                 │  3. SMILESルールエンジン    (v0.3)          │
                 │  4. LLM API                (v0.4)           │
                 └─────────────────────────────────────────────┘
                              │
                              ▼
                         HsPrediction
                          hs_code, confidence, notes, alternatives
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

> **注意**: テキスト入力として受け付けるのは **IUPACの系統名のみ** です。  
> 俗称・商品名（例：「苛性ソーダ」「カセイソーダ」）は対応していません（確実な変換が困難なため）。

---

## フィーチャーフラグ

| フラグ | 説明 | 追加される依存クレート |
|---|---|---|
| *(なし)* | ルールベースエンジンのみ | — |
| `pubchem` | PubChem API連携 (v0.2) | `reqwest`, `moka`, `governor` |
| `llm` | LLM API連携 (v0.4) | `reqwest`, `tokio` |
| `mock` | テスト用モックLLMクライアント | — |

`Cargo.toml` への追加方法:

```toml
[dependencies]
hs-predict = { version = "0.1", features = ["pubchem", "llm"] }
```

---

## 静的ルールテーブルの例

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

---

## サポートするRustバージョン（MSRV）

Rust **1.75** 以上（`async fn in traits` 安定化バージョン）。

---

## コントリビューション

バグ報告、静的ルールテーブルへの追加、PRを歓迎します。  
静的ルールテーブルへの新規エントリを追加する場合は、HS2022の品目表の該当章・号注を引用してください。

---

## ライセンス

以下のいずれかのライセンスのもとで提供されます:

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

どちらを選んでもかまいません。
