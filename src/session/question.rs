use serde::{Deserialize, Serialize};

/// The logical step a question belongs to.
///
/// Stored in [`ClassificationSession`](super::ClassificationSession) alongside
/// `current_question` so that `answer()` dispatches to the right state update
/// without inspecting language-specific prompt text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionStep {
    /// Step 1 — main product identifier.
    Identifier,
    /// Step 2 — is the product a mixture?
    IsMixture,
    /// Step 3a-i — number of mixture components.
    ComponentCount,
    /// Step 3a-ii — CAS / name of the n-th mixture component.
    ComponentIdentifier,
    /// Step 3a-iii — weight fraction of the current mixture component.
    ComponentFraction,
    /// Step 3b-i — physical form (solid / powder / liquid / …).
    PhysicalForm,
    /// Step 3b-ii — solution concentration (only asked after `PhysicalForm::Solution`).
    SolutionConcentration,
    /// Step 4 — intended end-use.
    IntendedUse,
    /// Step 5 — organic or inorganic (only when SMILES is unavailable).
    OrganicInorganic,
    /// Step 6 — functional groups (only for organic compounds without SMILES).
    FunctionalGroups,
}

/// セッションでユーザーに提示する質問
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Question {
    /// 自由テキスト入力
    Text {
        prompt: String,
        /// 入力例（省略可）
        example: Option<String>,
    },
    /// 選択肢から1つ選ぶ
    Choice {
        prompt: String,
        options: Vec<String>,
    },
    /// はい/いいえ
    YesNo {
        prompt: String,
    },
    /// 数値入力
    Number {
        prompt: String,
        unit: String,
        min: f64,
        max: f64,
    },
    /// 複数選択（官能基などの場合）
    MultiChoice {
        prompt: String,
        options: Vec<String>,
        /// 「わからない」選択肢を含むか
        include_unknown: bool,
    },
}

impl Question {
    pub fn prompt(&self) -> &str {
        match self {
            Question::Text { prompt, .. } => prompt,
            Question::Choice { prompt, .. } => prompt,
            Question::YesNo { prompt } => prompt,
            Question::Number { prompt, .. } => prompt,
            Question::MultiChoice { prompt, .. } => prompt,
        }
    }
}

/// ユーザーの回答
///
/// Serialized with adjacent tagging (`{ "kind": "text", "value": "..." }`)
/// so that primitive-containing variants (Text, Choice, …) round-trip correctly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Answer {
    /// テキスト入力の回答
    Text(String),
    /// Choice の選択インデックス (0-based)
    Choice(usize),
    /// YesNo の回答
    YesNo(bool),
    /// 数値入力
    Number(f64),
    /// MultiChoice の選択インデックスリスト
    MultiChoice(Vec<usize>),
    /// スキップ（任意項目のみ）
    Skip,
}

impl Answer {
    pub fn kind_name(&self) -> &'static str {
        match self {
            Answer::Text(_) => "text",
            Answer::Choice(_) => "choice",
            Answer::YesNo(_) => "yes_no",
            Answer::Number(_) => "number",
            Answer::MultiChoice(_) => "multi_choice",
            Answer::Skip => "skip",
        }
    }
}

/// 質問と回答のペア（セッション履歴用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAPair {
    pub question: Question,
    pub answer: Answer,
}

/// `answer()` メソッドの戻り値
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum SessionResult {
    /// 次の質問が必要
    NeedMoreInfo { next_question: Question },
    /// 十分な情報が集まった — パイプラインへ渡す準備完了
    Ready,
    /// ルールエンジンでは決定不能 — LLM へ委譲が必要
    RequiresLlm,
}
