//! Localised prompt strings for session questions.
//!
//! All user-visible text lives here so that adding a new language only
//! requires a new `match` arm, not a search across multiple files.

use crate::types::Language;

// ─── Question prompts ────────────────────────────────────────────────────────

/// Returns `(prompt, example)` for the main identifier question.
pub(crate) fn q_identifier(lang: Language) -> (String, String) {
    match lang {
        Language::En => (
            "Enter a chemical identifier (CAS number, IUPAC name, SMILES, or InChIKey)".into(),
            "e.g. 1310-73-2  or  sodium hydroxide  or  [Na+].[OH-]".into(),
        ),
        Language::Ja => (
            "化合物を特定する情報を入力してください（CAS番号 / IUPAC名 / SMILES / InChIKey のいずれか）".into(),
            "例: 1310-73-2　または　sodium hydroxide　または　[Na+].[OH-]".into(),
        ),
    }
}

/// Returns the "is this a mixture?" yes/no prompt.
pub(crate) fn q_is_mixture(lang: Language) -> String {
    match lang {
        Language::En => "Is this product a mixture of two or more compounds?".into(),
        Language::Ja => "この製品は 2 種類以上の化合物の混合物ですか？".into(),
    }
}

/// Returns `(prompt, unit)` for the component count question.
pub(crate) fn q_component_count(lang: Language) -> (String, String) {
    match lang {
        Language::En => ("How many components does it contain?".into(), "components".into()),
        Language::Ja => ("何種類の成分が含まれていますか？".into(), "種類".into()),
    }
}

/// Returns `(prompt, example)` for the n-th component identifier question (1-based).
pub(crate) fn q_component_identifier(lang: Language, n: usize) -> (String, String) {
    match lang {
        Language::En => (
            format!("Enter the CAS number or IUPAC name of component {n}"),
            "e.g. 7664-93-9  or  sulphuric acid".into(),
        ),
        Language::Ja => (
            format!("成分 {n} の CAS番号 または IUPAC名 を入力してください"),
            "例: 7664-93-9　または　sulphuric acid".into(),
        ),
    }
}

/// Returns `(prompt, unit)` for the component weight-fraction question.
pub(crate) fn q_component_fraction(lang: Language, name: &str) -> (String, String) {
    match lang {
        Language::En => (
            format!("Enter the weight fraction of \"{name}\" (enter 0 if unknown)"),
            "w/w%".into(),
        ),
        Language::Ja => (
            format!("「{name}」の重量割合 (w/w%) を入力してください（わからない場合は 0）"),
            "w/w%".into(),
        ),
    }
}

/// Returns `(prompt, options[])` for the physical-form choice question.
pub(crate) fn q_physical_form(lang: Language) -> (String, Vec<String>) {
    match lang {
        Language::En => (
            "Select the physical form of this product".into(),
            vec![
                "Solid (lumps, pellets, flakes, etc.)".into(),
                "Powder".into(),
                "Granules".into(),
                "Pure liquid (not a solution)".into(),
                "Solution (aqueous or in organic solvent)".into(),
                "Gas / vapour".into(),
                "Foil (metal foil, etc.)".into(),
                "Ingot (cast metal)".into(),
                "Unknown".into(),
            ],
        ),
        Language::Ja => (
            "この製品の物理的形状を選んでください".into(),
            vec![
                "固体（塊・ペレット・フレーク等）".into(),
                "粉末".into(),
                "粒状（造粒品）".into(),
                "純液体（溶液でない）".into(),
                "溶液（水溶液または有機溶媒溶液）".into(),
                "気体・蒸気".into(),
                "箔（金属箔等）".into(),
                "インゴット（鋳造品）".into(),
                "わからない".into(),
            ],
        ),
    }
}

/// Returns `(prompt, unit)` for the solution concentration question.
pub(crate) fn q_solution_concentration(lang: Language) -> (String, String) {
    match lang {
        Language::En => (
            "Enter the solute concentration (enter 0 if unknown)".into(),
            "w/w%".into(),
        ),
        Language::Ja => (
            "溶質の濃度を入力してください（わからない場合は 0 と入力）".into(),
            "w/w%".into(),
        ),
    }
}

/// Returns `(prompt, options[])` for the intended-use choice question.
pub(crate) fn q_intended_use(lang: Language) -> (String, Vec<String>) {
    match lang {
        Language::En => (
            "Select the primary intended use of this product".into(),
            vec![
                "Industrial (raw material, catalyst, solvent, etc.)".into(),
                "Pharmaceutical / medical".into(),
                "Agrochemical / fertiliser".into(),
                "Food / food additive".into(),
                "Cosmetic".into(),
                "Other / unknown".into(),
            ],
        ),
        Language::Ja => (
            "この製品の主な用途を選んでください".into(),
            vec![
                "工業用（原料・触媒・溶剤等）".into(),
                "医薬品・医療用".into(),
                "農薬・肥料".into(),
                "食品・食品添加物".into(),
                "化粧品".into(),
                "その他 / わからない".into(),
            ],
        ),
    }
}

/// Returns `(prompt, options[])` for the organic/inorganic choice question.
pub(crate) fn q_organic_inorganic(lang: Language) -> (String, Vec<String>) {
    match lang {
        Language::En => (
            "Is this compound organic or inorganic?".into(),
            vec![
                "Organic (carbon-backbone compound)".into(),
                "Inorganic".into(),
                "Unknown".into(),
            ],
        ),
        Language::Ja => (
            "この化合物は有機化合物ですか？".into(),
            vec![
                "有機化合物（炭素骨格を持つ）".into(),
                "無機化合物".into(),
                "わからない".into(),
            ],
        ),
    }
}

/// Returns `(prompt, options[])` for the functional-groups multi-choice question.
pub(crate) fn q_functional_groups(lang: Language) -> (String, Vec<String>) {
    match lang {
        Language::En => (
            "Select the main functional groups / structural features (multiple selection allowed)".into(),
            vec![
                "Carboxylic acid (-COOH)".into(),
                "Alcohol (-OH, aliphatic)".into(),
                "Phenol (-OH, aromatic)".into(),
                "Aldehyde (-CHO)".into(),
                "Ketone (C=O)".into(),
                "Amine (-NH2 / -NH-)".into(),
                "Amide (-CONH-)".into(),
                "Nitrile (-CN)".into(),
                "Halide (-Cl / -Br / -F / -I)".into(),
                "Ester (-COO-)".into(),
                "Aromatic ring".into(),
                "Other / unknown".into(),
            ],
        ),
        Language::Ja => (
            "主な官能基・化学構造を選んでください（複数選択可）".into(),
            vec![
                "カルボン酸 (-COOH)".into(),
                "アルコール (-OH、脂肪族)".into(),
                "フェノール (-OH、芳香族)".into(),
                "アルデヒド (-CHO)".into(),
                "ケトン (C=O)".into(),
                "アミン (-NH2 / -NH-)".into(),
                "アミド (-CONH-)".into(),
                "ニトリル (-CN)".into(),
                "ハロゲン化物 (-Cl / -Br / -F / -I)".into(),
                "エステル (-COO-)".into(),
                "芳香族環".into(),
                "その他 / わからない".into(),
            ],
        ),
    }
}

/// Returns `(prompt, example)` for the second component identifier question
/// (the "next component" branch in the mixture flow).
pub(crate) fn q_next_component_identifier(lang: Language, n: usize) -> (String, String) {
    match lang {
        Language::En => (
            format!("Enter the CAS number or IUPAC name of component {n}"),
            "e.g. 7732-18-5  or  water".into(),
        ),
        Language::Ja => (
            format!("成分 {n} の CAS番号 または IUPAC名 を入力してください"),
            "例: 7732-18-5　または　water".into(),
        ),
    }
}
