//! Prompt builder for LLM-based HS code classification.
//!
//! [`PromptBuilder`] converts a [`ProductDescription`](crate::types::ProductDescription)
//! into a ready-to-send [`LlmPrompt`](super::LlmPrompt) in English or Japanese.

use crate::types::{Language, PhysicalForm, ProductDescription};
use super::LlmPrompt;

// ─────────────────────────────────────────────────────────────────────────────
// PromptBuilder
// ─────────────────────────────────────────────────────────────────────────────

/// Builds the system and user prompt texts from a [`ProductDescription`].
///
/// # Example
/// ```rust
/// # #[cfg(feature = "llm")]
/// # {
/// use hs_predict::llm::PromptBuilder;
/// use hs_predict::types::{ProductDescription, SubstanceIdentifier, PhysicalForm, Language};
///
/// let product = ProductDescription {
///     identifier: SubstanceIdentifier::from_cas("64-19-7"),
///     physical_form: Some(PhysicalForm::Liquid),
///     purity_pct: Some(99.8),
///     purity_type: None,
///     mixture_components: None,
///     intended_use: None,
///     additional_context: None,
/// };
///
/// let prompt = PromptBuilder::new().build(&product);
/// assert!(prompt.system_text.contains("HS 2022"));
/// assert!(prompt.user_text.contains("64-19-7"));
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct PromptBuilder {
    language: Language,
}

impl PromptBuilder {
    /// Create a new builder that emits English prompts.
    pub fn new() -> Self {
        Self { language: Language::En }
    }

    /// Set the output language.
    pub fn with_language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    /// Build the [`LlmPrompt`] from the given product description.
    pub fn build(&self, product: &ProductDescription) -> LlmPrompt {
        let smiles_analysis = product
            .identifier
            .smiles
            .as_deref()
            .and_then(crate::smiles::classify_smiles);

        let system_text = match self.language {
            Language::En => self.system_text_en(),
            Language::Ja => self.system_text_ja(),
        };

        let user_text = match self.language {
            Language::En => self.user_text_en(product, smiles_analysis.as_ref()),
            Language::Ja => self.user_text_ja(product, smiles_analysis.as_ref()),
        };

        LlmPrompt {
            system_text,
            user_text,
            smiles_analysis,
        }
    }

    // ─── System prompts ───────────────────────────────────────────────

    fn system_text_en(&self) -> String {
        r#"You are an expert customs classification specialist with deep knowledge of the
Harmonized System (HS) 2022 nomenclature, particularly Chapters 28 and 29 for
chemical products.

Your task is to assign a six-digit HS 2022 code to the chemical product described
in the user message.

## Output format

Respond with **only** a JSON object — no prose, no markdown:

```json
{
  "hs_code":    "<6 ASCII digits, no dots>",
  "confidence": <float 0.0–1.0>,
  "rationale":  "<1–3 sentences explaining the classification>",
  "alternatives": [
    { "hs_code": "<6 digits>", "confidence": <float>, "reason": "<brief>" }
  ]
}
```

`alternatives` may be an empty array `[]`.

## Confidence guide

| Score | Meaning |
|-------|---------|
| ≥ 0.90 | Certain of the 6-digit sub-heading |
| ≥ 0.70 | Certain of the 4-digit heading, sub-heading uncertain |
| ≥ 0.50 | Chapter correct, heading uncertain |
| < 0.50 | Significant uncertainty — classify to the most likely heading |

## Rules

- Use HS 2022 edition.
- If a SMILES-derived heading hint is provided, treat it as a cross-check, not
  authoritative — rule 1 of HS Explanatory Notes takes precedence over chemical
  structure alone.
- Always verify Chapter Notes and Section Notes before finalising.
- For mixtures, classify by the component that gives the mixture its essential
  character (GRI 3b) unless a specific mixture heading applies.
"#.to_string()
    }

    fn system_text_ja(&self) -> String {
        r#"あなたは輸出入通関の専門家であり、HS 2022 品目表（特に第28類・第29類の化学品）に
精通しています。

ユーザーメッセージに記載された化学品に対して、6桁の HS 2022 コードを付与してください。

## 出力形式

**JSON オブジェクトのみ**を返答してください（文章・マークダウン不要）：

```json
{
  "hs_code":    "<6桁の数字、ドットなし>",
  "confidence": <0.0〜1.0 の小数>,
  "rationale":  "<分類根拠を1〜3文で>",
  "alternatives": [
    { "hs_code": "<6桁>", "confidence": <小数>, "reason": "<簡潔な理由>" }
  ]
}
```

`alternatives` は空配列 `[]` でも可。

## 信頼度の目安

| スコア | 意味 |
|--------|------|
| ≥ 0.90 | 6桁の細分まで確実 |
| ≥ 0.70 | 4桁の号まで確実、細分は不確実 |
| ≥ 0.50 | 類は正しいが号が不確実 |
| < 0.50 | 大きな不確実性あり — 最も可能性の高い号に分類 |

## ルール

- HS 2022年版を使用すること。
- SMILES由来のヘッディングヒントが提供された場合は参考情報として扱い、
  HS解説書の通則1を優先すること。
- 分類確定前に類注および部注を確認すること。
- 混合物の場合、特定の混合物号がない限り、本質的特性を与える成分で分類（通則3(b)）。
"#.to_string()
    }

    // ─── User prompts ─────────────────────────────────────────────────

    fn user_text_en(
        &self,
        product: &ProductDescription,
        smiles_analysis: Option<&crate::smiles::SmilesClassification>,
    ) -> String {
        let mut parts: Vec<String> = Vec::new();

        parts.push("## Product to classify".to_string());
        parts.push(String::new());

        // Identifiers
        let id = &product.identifier;
        if let Some(ref cas) = id.cas {
            parts.push(format!("- **CAS**: {}", cas));
        }
        if let Some(ref iupac) = id.iupac_name {
            parts.push(format!("- **IUPAC name**: {}", iupac));
        }
        if let Some(ref smiles) = id.smiles {
            parts.push(format!("- **SMILES**: {}", smiles));
        }
        if let Some(ref inchi) = id.inchi {
            parts.push(format!("- **InChI**: {}", inchi));
        }
        if let Some(ref inchikey) = id.inchi_key {
            parts.push(format!("- **InChIKey**: {}", inchikey));
        }

        // Physical form
        if let Some(ref form) = product.physical_form {
            parts.push(format!("- **Physical form**: {}", physical_form_en(form)));
        }

        // Purity
        if let Some(purity) = product.purity_pct {
            parts.push(format!("- **Purity**: {:.1}%", purity));
        }

        // Intended use
        if let Some(ref use_) = product.intended_use {
            parts.push(format!("- **Intended use**: {:?}", use_));
        }

        // Mixture components
        if let Some(ref comps) = product.mixture_components {
            parts.push("- **Mixture components**:".to_string());
            for c in comps {
                let frac = c
                    .weight_fraction_pct
                    .map(|f| format!(" ({:.1}% w/w)", f))
                    .unwrap_or_default();
                let name = c.substance.cas.as_deref()
                    .or(c.substance.iupac_name.as_deref())
                    .unwrap_or("unknown");
                parts.push(format!("  - {}{}", name, frac));
            }
        }

        // Additional context
        if let Some(ref ctx) = product.additional_context {
            parts.push(format!("- **Additional context**: {}", ctx));
        }

        // SMILES analysis hint
        if let Some(analysis) = smiles_analysis {
            parts.push(String::new());
            parts.push("## SMILES pre-analysis hint".to_string());
            parts.push(String::new());
            parts.push(format!(
                "- **Organic class**: {}",
                format!("{:?}", analysis.organic_class)
            ));
            if !analysis.functional_groups.is_empty() {
                let groups: Vec<&str> = analysis
                    .functional_groups
                    .iter()
                    .map(|g| g.label())
                    .collect();
                parts.push(format!("- **Functional groups detected**: {}", groups.join(", ")));
            }
            let hint = &analysis.heading_hint;
            if let Some(heading) = hint.heading {
                parts.push(format!(
                    "- **Heading hint**: {}.{:02} ({}, confidence {:.2})",
                    heading / 100,
                    heading % 100,
                    hint.rationale,
                    hint.confidence
                ));
            } else {
                parts.push(format!(
                    "- **Chapter hint**: Ch.{:02} (confidence {:.2})",
                    hint.chapter, hint.confidence
                ));
            }
            parts.push(String::new());
            parts.push(
                "_This hint is derived from SMILES pattern matching and is provided for \
                 cross-checking only. Apply the HS Explanatory Notes authoritatively._"
                    .to_string(),
            );
        }

        parts.join("\n")
    }

    fn user_text_ja(
        &self,
        product: &ProductDescription,
        smiles_analysis: Option<&crate::smiles::SmilesClassification>,
    ) -> String {
        let mut parts: Vec<String> = Vec::new();

        parts.push("## 分類対象品目".to_string());
        parts.push(String::new());

        let id = &product.identifier;
        if let Some(ref cas) = id.cas {
            parts.push(format!("- **CAS番号**: {}", cas));
        }
        if let Some(ref iupac) = id.iupac_name {
            parts.push(format!("- **IUPAC名**: {}", iupac));
        }
        if let Some(ref smiles) = id.smiles {
            parts.push(format!("- **SMILES**: {}", smiles));
        }
        if let Some(ref inchi) = id.inchi {
            parts.push(format!("- **InChI**: {}", inchi));
        }
        if let Some(ref inchikey) = id.inchi_key {
            parts.push(format!("- **InChIKey**: {}", inchikey));
        }

        if let Some(ref form) = product.physical_form {
            parts.push(format!("- **物理的形状**: {}", physical_form_ja(form)));
        }

        if let Some(purity) = product.purity_pct {
            parts.push(format!("- **純度**: {:.1}%", purity));
        }

        if let Some(ref use_) = product.intended_use {
            parts.push(format!("- **用途**: {:?}", use_));
        }

        if let Some(ref comps) = product.mixture_components {
            parts.push("- **混合成分**:".to_string());
            for c in comps {
                let frac = c
                    .weight_fraction_pct
                    .map(|f| format!(" ({:.1}% w/w)", f))
                    .unwrap_or_default();
                let name = c.substance.cas.as_deref()
                    .or(c.substance.iupac_name.as_deref())
                    .unwrap_or("不明");
                parts.push(format!("  - {}{}", name, frac));
            }
        }

        if let Some(ref ctx) = product.additional_context {
            parts.push(format!("- **補足情報**: {}", ctx));
        }

        if let Some(analysis) = smiles_analysis {
            parts.push(String::new());
            parts.push("## SMILES 事前解析ヒント".to_string());
            parts.push(String::new());
            parts.push(format!(
                "- **有機/無機区分**: {}",
                format!("{:?}", analysis.organic_class)
            ));
            if !analysis.functional_groups.is_empty() {
                let groups: Vec<&str> = analysis
                    .functional_groups
                    .iter()
                    .map(|g| g.label())
                    .collect();
                parts.push(format!("- **検出官能基**: {}", groups.join("、")));
            }
            let hint = &analysis.heading_hint;
            if let Some(heading) = hint.heading {
                parts.push(format!(
                    "- **号ヒント**: {}.{:02}（{}、信頼度 {:.2}）",
                    heading / 100,
                    heading % 100,
                    hint.rationale,
                    hint.confidence
                ));
            } else {
                parts.push(format!(
                    "- **類ヒント**: 第{:02}類（信頼度 {:.2}）",
                    hint.chapter, hint.confidence
                ));
            }
            parts.push(String::new());
            parts.push(
                "_このヒントはSMILESパターンマッチングによるもので、参考情報です。\
                 HS解説書を正式な根拠として適用してください。_"
                    .to_string(),
            );
        }

        parts.join("\n")
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn physical_form_en(form: &PhysicalForm) -> &'static str {
    match form {
        PhysicalForm::Solid => "Solid",
        PhysicalForm::Powder { .. } => "Powder",
        PhysicalForm::Granules => "Granules",
        PhysicalForm::Liquid => "Liquid",
        PhysicalForm::Solution { .. } => "Solution",
        PhysicalForm::Gas => "Gas",
        PhysicalForm::Foil { .. } => "Foil",
        PhysicalForm::Ingot => "Ingot",
        PhysicalForm::Unknown => "Unknown",
    }
}

fn physical_form_ja(form: &PhysicalForm) -> &'static str {
    match form {
        PhysicalForm::Solid => "固体",
        PhysicalForm::Powder { .. } => "粉末",
        PhysicalForm::Granules => "顆粒",
        PhysicalForm::Liquid => "液体",
        PhysicalForm::Solution { .. } => "溶液",
        PhysicalForm::Gas => "気体",
        PhysicalForm::Foil { .. } => "箔",
        PhysicalForm::Ingot => "インゴット",
        PhysicalForm::Unknown => "不明",
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ProductDescription, SubstanceIdentifier};

    fn acetic_acid() -> ProductDescription {
        ProductDescription {
            identifier: SubstanceIdentifier {
                cas: Some("64-19-7".to_string()),
                iupac_name: Some("acetic acid".to_string()),
                smiles: Some("CC(O)=O".to_string()),
                inchi: None,
                inchi_key: None,
                cid: None,
            },
            physical_form: Some(PhysicalForm::Liquid),
            purity_pct: Some(99.5),
            purity_type: None,
            mixture_components: None,
            intended_use: None,
            additional_context: None,
        }
    }

    #[test]
    fn en_system_prompt_contains_hs_2022() {
        let p = PromptBuilder::new().build(&acetic_acid());
        assert!(p.system_text.contains("HS 2022"));
    }

    #[test]
    fn en_user_text_contains_cas() {
        let p = PromptBuilder::new().build(&acetic_acid());
        assert!(p.user_text.contains("64-19-7"));
    }

    #[test]
    fn en_user_text_contains_purity() {
        let p = PromptBuilder::new().build(&acetic_acid());
        assert!(p.user_text.contains("99.5"));
    }

    #[test]
    fn en_user_text_contains_smiles_hint() {
        let p = PromptBuilder::new().build(&acetic_acid());
        // acetic acid SMILES → carboxylic acid → heading 29.15
        assert!(p.user_text.contains("Heading hint") || p.user_text.contains("heading hint")
            || p.user_text.contains("SMILES pre-analysis"));
    }

    #[test]
    fn smiles_analysis_populated_when_smiles_present() {
        let p = PromptBuilder::new().build(&acetic_acid());
        assert!(p.smiles_analysis.is_some());
    }

    #[test]
    fn smiles_analysis_none_when_no_smiles() {
        let product = ProductDescription {
            identifier: SubstanceIdentifier::from_cas("64-19-7"),
            physical_form: None,
            purity_pct: None,
            purity_type: None,
            mixture_components: None,
            intended_use: None,
            additional_context: None,
        };
        let p = PromptBuilder::new().build(&product);
        assert!(p.smiles_analysis.is_none());
    }

    #[test]
    fn ja_system_prompt_contains_hs_2022_ja() {
        let p = PromptBuilder::new()
            .with_language(Language::Ja)
            .build(&acetic_acid());
        assert!(p.system_text.contains("HS 2022"));
    }

    #[test]
    fn ja_user_text_contains_cas() {
        let p = PromptBuilder::new()
            .with_language(Language::Ja)
            .build(&acetic_acid());
        assert!(p.user_text.contains("64-19-7"));
    }

    #[test]
    fn mixture_components_listed() {
        use crate::types::MixtureComponent;
        let product = ProductDescription {
            identifier: SubstanceIdentifier::from_cas("7732-18-5"),
            physical_form: Some(PhysicalForm::Solution {
                concentration_pct_ww: Some(30.0),
                solvent: None,
            }),
            purity_pct: None,
            purity_type: None,
            mixture_components: Some(vec![
                MixtureComponent {
                    substance: SubstanceIdentifier::from_cas("1310-73-2"),
                    weight_fraction_pct: Some(30.0),
                    volume_fraction_pct: None,
                    is_solvent: false,
                },
            ]),
            intended_use: None,
            additional_context: None,
        };
        let p = PromptBuilder::new().build(&product);
        assert!(p.user_text.contains("1310-73-2"));
        assert!(p.user_text.contains("30.0"));
    }
}
