//! Organic / inorganic detection and functional group detection from SMILES.
//!
//! Detection is based on substring pattern matching against canonical SMILES
//! (as returned by PubChem). It is intentionally approximate — results carry
//! a confidence of ≤ 0.70 and are used only as heading-level hints.
//!
//! # Priority order
//! Groups are checked in decreasing specificity so that more specific patterns
//! take precedence (e.g. anhydride before ester before carboxylic acid).

use crate::types::OrganicInorganic;
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// FunctionalGroup enum
// ─────────────────────────────────────────────────────────────────────────────

/// Functional group category detectable from a SMILES string.
///
/// The 20 groups cover the main HS Chapter 29 classification criteria
/// for organic chemicals plus the organic/inorganic distinction used
/// for Chapter 28.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionalGroup {
    /// –C(=O)–O–C(=O)– (acid anhydride).
    Anhydride,
    /// –N=C=O (isocyanate or isothiocyanate N=C=S).
    Isocyanate,
    /// –C≡N (nitrile / cyanide).
    Nitrile,
    /// –[N+](=O)[O–] nitro group.
    Nitro,
    /// Three-membered ring containing O (epoxide).
    Epoxide,
    /// –S(=O)(=O)–OH sulphonic acid.
    SulphonicAcid,
    /// P=O or P–O (phosphate / phosphonate ester).
    Phosphate,
    /// –C(=O)–NH₂ / –NHC(=O)– amide.
    Amide,
    /// –C(=O)–O–C ester (not anhydride).
    Ester,
    /// –C(=O)–OH carboxylic acid.
    CarboxylicAcid,
    /// –CHO terminal aldehyde.
    Aldehyde,
    /// –C(=O)– flanked by two C atoms (ketone).
    Ketone,
    /// Phenolic –OH on aromatic ring.
    Phenol,
    /// –SH thiol (mercaptan).
    Thiol,
    /// C–S–C thioether / sulphide.
    Sulphide,
    /// Aliphatic –C–OH alcohol.
    Alcohol,
    /// C–O–C ether (not ester, not epoxide).
    Ether,
    /// Primary, secondary, or tertiary amine –NHₓ (not amide).
    Amine,
    /// C–F / C–Cl / C–Br / C–I organic halide.
    Halide,
    /// Aromatic ring (any aromatic atom present).
    AromaticRing,
}

impl FunctionalGroup {
    /// Short display label for notes and logging.
    pub fn label(self) -> &'static str {
        match self {
            Self::Anhydride => "Anhydride",
            Self::Isocyanate => "Isocyanate",
            Self::Nitrile => "Nitrile",
            Self::Nitro => "Nitro",
            Self::Epoxide => "Epoxide",
            Self::SulphonicAcid => "SulphonicAcid",
            Self::Phosphate => "Phosphate",
            Self::Amide => "Amide",
            Self::Ester => "Ester",
            Self::CarboxylicAcid => "CarboxylicAcid",
            Self::Aldehyde => "Aldehyde",
            Self::Ketone => "Ketone",
            Self::Phenol => "Phenol",
            Self::Thiol => "Thiol",
            Self::Sulphide => "Sulphide",
            Self::Alcohol => "Alcohol",
            Self::Ether => "Ether",
            Self::Amine => "Amine",
            Self::Halide => "Halide",
            Self::AromaticRing => "AromaticRing",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Organic / inorganic classification
// ─────────────────────────────────────────────────────────────────────────────

/// Determine whether a SMILES string represents an organic, inorganic,
/// or organometallic compound.
///
/// Uses the chemical definition: *organic* = contains at least one carbon atom
/// that is not in a purely inorganic context (CO₂, CO, CS₂, carbonate, cyanide
/// as free ion).
pub fn classify_organic(smiles: &str) -> OrganicInorganic {
    // No carbon → definitely inorganic
    if !smiles.chars().any(|c| c == 'C' || c == 'c') {
        return OrganicInorganic::Inorganic;
    }

    // Exact-match known simple inorganic carbon compounds
    let normalised = smiles.replace(' ', "");
    let inorganic_exact: &[&str] = &[
        "O=C=O",       // CO₂
        "[O-]C(=O)[O-]", // carbonate ion
        "[O-]C([O-])=O",
        "[C-]#[O+]",   // CO
        "[C+]#[O-]",
        "S=C=S",       // CS₂
        "[C-]#N",      // cyanide ion
        "[N+]#[C-]",
        "C(=O)([O-])[O-]", // carbonate
    ];
    if inorganic_exact.iter().any(|p| normalised == *p) {
        return OrganicInorganic::Inorganic;
    }

    // Check multi-component SMILES (dot-separated): each fragment independently
    // A compound is organometallic if any fragment has a direct metal–C bond.
    let metal_symbols: &[&str] = &[
        "[Fe]", "[Co]", "[Ni]", "[Cr]", "[Mn]", "[Mo]", "[W]",
        "[Ti]", "[V]",  "[Ru]", "[Rh]", "[Pd]", "[Os]", "[Ir]",
        "[Pt]", "[Zn]", "[Al]", "[Pb]", "[Sn]", "[Hg]", "[Tl]",
    ];
    // Organometallic: metal atom directly bonded to carbon in SMILES notation
    // i.e. the metal symbol is followed or preceded by C/c (with no space or [)
    for metal in metal_symbols {
        if smiles.contains(metal) {
            // Check if this metal is bonded to C in the SMILES graph.
            // Heuristic: metal symbol immediately adjacent to C or c in the string.
            let idx = smiles.find(metal).unwrap_or(usize::MAX);
            let after = smiles.get(idx + metal.len()..).unwrap_or("");
            let before = smiles.get(..idx).unwrap_or("");
            let bonded = after.starts_with('C')
                || after.starts_with('c')
                || before.ends_with('C')
                || before.ends_with('c');
            if bonded {
                return OrganicInorganic::Organometallic;
            }
        }
    }

    OrganicInorganic::Organic
}

// ─────────────────────────────────────────────────────────────────────────────
// Functional group detection
// ─────────────────────────────────────────────────────────────────────────────

/// Detect functional groups present in a SMILES string.
///
/// The detection uses substring pattern matching against both the
/// canonical and common alternative SMILES representations.
/// Groups are returned in detection priority order (most specific first).
///
/// # Limitations
/// - Does not perform full SMILES parsing; edge cases may be missed.
/// - Designed primarily for PubChem canonical SMILES.
/// - Confidences are capped at ≤ 0.70 due to these limitations.
pub fn detect_functional_groups(smiles: &str) -> Vec<FunctionalGroup> {
    let mut groups: Vec<FunctionalGroup> = Vec::new();

    // Helper: returns true if any of `patterns` is a substring of `smiles`.
    let any = |patterns: &[&str]| -> bool { patterns.iter().any(|p| smiles.contains(p)) };

    // ── 1. Anhydride (check before ester and acid) ────────────────────────
    // Linear anhydride: C(=O)OC(=O) (e.g. acetic anhydride: CC(=O)OC(=O)C)
    // Cyclic anhydride: O=C[digit]OC(=O) (e.g. phthalic: O=C1OC(=O)c2ccccc21)
    let cyclic_anhydride = (1u8..=9).any(|n| {
        smiles.contains(&format!("O=C{}OC(=O)", n))
    });
    if smiles.contains("C(=O)OC(=O)") || cyclic_anhydride {
        groups.push(FunctionalGroup::Anhydride);
    }

    // ── 2. Isocyanate ─────────────────────────────────────────────────────
    if any(&["N=C=O", "O=C=N"]) {
        groups.push(FunctionalGroup::Isocyanate);
    }

    // ── 3. Nitrile ────────────────────────────────────────────────────────
    if any(&["C#N", "N#C"]) {
        groups.push(FunctionalGroup::Nitrile);
    }

    // ── 4. Nitro ──────────────────────────────────────────────────────────
    // PubChem canonical writes the double-bond O before N: O=[N+]([O-])
    if any(&[
        "O=[N+]([O-])", // PubChem canonical (nitrobenzene, TNT, etc.)
        "[N+](=O)[O-]", // alternative bracket form
        "N(=O)=O",
        "[N+]([O-])=O",
        "[N+](=O)([O-])",
    ]) {
        groups.push(FunctionalGroup::Nitro);
    }

    // ── 5. Epoxide (3-membered ring with O) ───────────────────────────────
    // PubChem canonical for ethylene oxide: C1CO1 (C-C-O ring).
    // Also handle C1OC1 (alternative) and stereocentres.
    if any(&[
        "C1CO1",           // ethylene oxide / PubChem canonical
        "C1OC1",           // alternative ring ordering
        "[C@@H]1O[C@H]1",  // stereo epoxide
        "[C@H]1O[C@@H]1",
    ]) {
        groups.push(FunctionalGroup::Epoxide);
    }

    // ── 6. Sulphonic acid ─────────────────────────────────────────────────
    if any(&["S(=O)(=O)O", "S(=O)(=O)[OH]", "S(O)(=O)=O", "[S](=O)(=O)O"]) {
        groups.push(FunctionalGroup::SulphonicAcid);
    }

    // ── 7. Phosphate / phosphonate ────────────────────────────────────────
    if smiles.contains('P')
        && any(&["P(=O)(O)", "P(=O)([O", "P(O)(O)", "P([OH])", "OP(=O)", "P(=O)O"])
    {
        groups.push(FunctionalGroup::Phosphate);
    }

    // ── 8. Amide (before amine) ───────────────────────────────────────────
    // Canonical: NC(=O), NC(C...)=O, C(N)=O, C(=O)N, C(=O)[NH
    if any(&[
        "NC(=O)", "NC(C", // NC(C...)=O  — amide N before carbonyl-C
        "C(N)=O", "C(=O)N", "C(=O)[NH", "[NH]C(=O)", "[NH2]C(=O)",
        "N)=O",   // -N)=O terminal amide
    ]) {
        // Exclude isocyanate and nitrile (already tagged)
        let has_iso = groups.contains(&FunctionalGroup::Isocyanate);
        let has_nitrile = groups.contains(&FunctionalGroup::Nitrile);
        if !has_iso && !has_nitrile {
            groups.push(FunctionalGroup::Amide);
        }
    }

    // ── 9. Ester (before carboxylic acid) ─────────────────────────────────
    // Canonical: OC(C...)=O (ester O before carbonyl-C), C(=O)OC
    let has_anhydride = groups.contains(&FunctionalGroup::Anhydride);
    if !has_anhydride
        && any(&[
            "OC(C)=O", "OC(=O)C", "C(=O)OC", "C(=O)Oc",  // common ester patterns
            "OC(CC", "OC(c",  // aromatic/branched esters
        ])
    {
        groups.push(FunctionalGroup::Ester);
    }

    // ── 10. Carboxylic acid ────────────────────────────────────────────────
    // After ester to avoid false positives
    let has_ester = groups.contains(&FunctionalGroup::Ester);
    if !has_ester && !has_anhydride {
        // Acid patterns: C(=O)O terminal, C(O)=O, OC(=O) at boundaries
        // In canonical SMILES: acetic acid = CC(=O)O (O is terminal)
        let has_acid_pattern = any(&[
            "C(=O)O",    // acetic acid: CC(=O)O — O terminal
            "C(O)=O",    // alternative writing
            "C(=O)[OH]", // explicit H on O
        ]);
        // Exclude if the pattern belongs to carbonate or similar
        if has_acid_pattern {
            groups.push(FunctionalGroup::CarboxylicAcid);
        }
    }

    // ── 11. Aldehyde ──────────────────────────────────────────────────────
    // Terminal C=O with no second C on the carbonyl C
    // Canonical: CC=O, O=Cc..., [CH]=O
    let has_higher_carbonyl = groups.iter().any(|g| {
        matches!(
            g,
            FunctionalGroup::Amide
                | FunctionalGroup::Ester
                | FunctionalGroup::CarboxylicAcid
                | FunctionalGroup::Anhydride
        )
    });
    if !has_higher_carbonyl {
        let aldehyde = smiles.ends_with("C=O")
            || smiles.ends_with("[CH]=O")
            || smiles.starts_with("O=C")  // e.g. O=Cc1ccccc1 (benzaldehyde)
            || any(&["[CH]=O", "[CHO]"]);
        if aldehyde {
            groups.push(FunctionalGroup::Aldehyde);
        }
    }

    // ── 12. Ketone ────────────────────────────────────────────────────────
    // Carbonyl C with C on both sides; canonical: CC(C)=O, CC(CC)=O
    if !has_higher_carbonyl {
        let has_aldehyde = groups.contains(&FunctionalGroup::Aldehyde);
        if !has_aldehyde
            && any(&[
                "C(C)=O",  // CC(C)=O acetone, CC(CC)=O 2-butanone
                "C(CC)=O", "C(CCC)=O",
                "C(c)=O",  // aryl ketone: C(c1...)=O
                "c(=O)C",  // aromatic ketone
                "C(=O)C",  // alternative form: CC(=O)CC
            ])
        {
            groups.push(FunctionalGroup::Ketone);
        }
    }

    // ── 13. Phenol ────────────────────────────────────────────────────────
    if any(&[
        "c1ccccc1O", "Oc1ccccc1",
        "c(O)",      // aromatic C-OH inline
        "c([OH])",   // explicit
        "Oc1cc", "Oc1ccc", "c1cc(O)", "c1ccc(O)",
    ]) {
        groups.push(FunctionalGroup::Phenol);
    }

    // ── 14. Thiol ─────────────────────────────────────────────────────────
    // Canonical: [SH] explicit, or CS at end of string
    if any(&["[SH]", "C[SH]", "c[SH]"])
        || smiles.ends_with("CS")
        || smiles.ends_with("cS")
    {
        groups.push(FunctionalGroup::Thiol);
    }

    // ── 15. Sulphide (after thiol and sulphonic acid) ──────────────────────
    let has_sulphonic = groups.contains(&FunctionalGroup::SulphonicAcid);
    let has_thiol = groups.contains(&FunctionalGroup::Thiol);
    if !has_sulphonic
        && !has_thiol
        && smiles.contains('S')
        && any(&["CSC", "cSC", "CSc", "cSc", "C(S)C"])
    {
        groups.push(FunctionalGroup::Sulphide);
    }

    // ── 16. Alcohol ───────────────────────────────────────────────────────
    // Aliphatic C-OH: [OH] explicit, terminal O in chain, or (O) pendant
    let has_phenol = groups.contains(&FunctionalGroup::Phenol);
    let has_acid = groups.contains(&FunctionalGroup::CarboxylicAcid);
    let has_ester2 = groups.contains(&FunctionalGroup::Ester);
    let has_anhydride2 = groups.contains(&FunctionalGroup::Anhydride);
    // Also guard against aldehyde: "CC=O" ends with "O" but is not an alcohol.
    let has_aldehyde_grp = groups.contains(&FunctionalGroup::Aldehyde);
    if !has_phenol && !has_acid && !has_ester2 && !has_anhydride2 && !has_aldehyde_grp {
        let alcohol = any(&["[OH]", "C[OH]"])
            || smiles.ends_with("CO")
            || smiles.ends_with("CCO")
            || smiles.ends_with("O")  // generic terminal O (e.g. CCO = ethanol)
            || any(&["C(O)", "C([OH])"]);
        if alcohol {
            groups.push(FunctionalGroup::Alcohol);
        }
    }

    // ── 17. Ether ─────────────────────────────────────────────────────────
    // C-O-C not ester, not epoxide, not acid anhydride
    let has_epoxide = groups.contains(&FunctionalGroup::Epoxide);
    let has_ester3 = groups.contains(&FunctionalGroup::Ester);
    let has_acid2 = groups.contains(&FunctionalGroup::CarboxylicAcid);
    if !has_epoxide && !has_ester3 && !has_acid2 && !has_anhydride
        && any(&["COC", "cOC", "COc", "cOc"]) {
        groups.push(FunctionalGroup::Ether);
    }

    // ── 18. Amine ─────────────────────────────────────────────────────────
    // N not in amide, nitrile, nitro
    let has_amide = groups.contains(&FunctionalGroup::Amide);
    let has_nitrile = groups.contains(&FunctionalGroup::Nitrile);
    let has_nitro = groups.contains(&FunctionalGroup::Nitro);
    if smiles.contains('N')
        && !has_nitrile
        && !has_nitro
    {
        // Look for amine patterns not adjacent to a carbonyl
        let amine = any(&[
            "CN", "NC", "[NH2]", "[NH3+]", "[NH]", "cN", "Nc",
        ]);
        // If amide already detected, only add amine if there's a free amine too
        if amine && (!has_amide || any(&["[NH2]", "[NH3+]", "CN(", "N(C)C"])) {
            groups.push(FunctionalGroup::Amine);
        }
    }

    // ── 19. Halide ────────────────────────────────────────────────────────
    if any(&[
        "CF", "CCl", "CBr", "CI",
        "Fc", "Clc", "Brc", "Ic",
        "[F]", "[Cl]", "[Br]", "[I]",
        "c[F]", "c[Cl]", "c[Br]", "c[I]",
        "CF3", "CCl3", "CHF", "CHCl", "CHBr",
    ]) {
        groups.push(FunctionalGroup::Halide);
    }

    // ── 20. Aromatic ring (last — lowest priority) ────────────────────────
    if smiles.chars().any(|c| matches!(c, 'c' | 'n' | 'o' | 's' | 'p')) {
        groups.push(FunctionalGroup::AromaticRing);
    }

    groups
}

// ─────────────────────────────────────────────────────────────────────────────
// Structural feature extraction
// ─────────────────────────────────────────────────────────────────────────────

/// Atom-count and connectivity properties extracted from a SMILES string.
///
/// These supplement functional-group detection and are used by
/// [`crate::smiles::chapter_map::map_to_subheading`] to resolve
/// 4-digit HS headings to 6-digit subheadings.
///
/// Analysis is heuristic and designed for PubChem canonical SMILES.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructuralFeatures {
    /// Total carbon atom count (uppercase C + aromatic c, excluding Cl).
    pub carbon_count: u32,
    /// Estimated hydroxyl (–OH) group count.
    ///
    /// For carboxylic acids this includes the acid –OH (one per –COOH).
    /// Use `hydroxyl_count.saturating_sub(1)` when `CarboxylicAcid` is
    /// in the detected functional groups to get the extra alcohol –OH count.
    pub hydroxyl_count: u32,
    /// Number of C=O (carbonyl) groups (ketone, aldehyde, ester, acid, etc.).
    pub carbonyl_count: u32,
    /// `true` when the SMILES contains a ring-closure digit outside brackets.
    pub has_ring: bool,
    /// `true` when lowercase aromatic-carbon atoms (`c`) are present.
    pub has_aromatic_ring: bool,
    /// `true` when a C=C aliphatic double bond is present.
    pub has_cc_double_bond: bool,
    /// `true` when a halogen substituent (F, Cl, Br, I) is present.
    pub has_halogen: bool,
}

/// Extract structural features from a canonical SMILES string.
///
/// The analysis is approximate.  Use together with [`detect_functional_groups`]
/// to narrow 4-digit HS headings down to 6-digit subheadings.
pub fn detect_structural_features(smiles: &str) -> StructuralFeatures {
    StructuralFeatures {
        carbon_count:      count_carbons(smiles),
        hydroxyl_count:    count_hydroxyls(smiles),
        carbonyl_count:    smiles.matches("=O").count() as u32,
        has_ring:          ring_present(smiles),
        has_aromatic_ring: smiles.contains('c'),
        has_cc_double_bond: cc_double_bond_present(smiles),
        has_halogen: smiles.contains('F')
            || smiles.contains("Cl")
            || smiles.contains("Br")
            || (smiles.contains('I') && !smiles.contains("In")),
    }
}

/// Count carbon atoms in a SMILES string.
/// Handles bracket atoms (`[13C]`, `[CH2]`) and skips `Cl` (chlorine).
fn count_carbons(smiles: &str) -> u32 {
    let mut count = 0u32;
    let mut chars = smiles.chars().peekable();
    let mut in_bracket = false;
    let mut bracket_buf = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            '[' => {
                in_bracket = true;
                bracket_buf.clear();
            }
            ']' if in_bracket => {
                in_bracket = false;
                // Strip leading isotope digits then inspect the atom symbol.
                let sym = bracket_buf.trim_start_matches(|c: char| c.is_ascii_digit());
                if sym.starts_with('C') || sym.starts_with('c') {
                    count += 1;
                }
            }
            c if in_bracket => bracket_buf.push(c),
            'C' => {
                if chars.peek() == Some(&'l') {
                    chars.next(); // Cl = chlorine, not carbon
                } else {
                    count += 1;
                }
            }
            'c' => count += 1,
            _ => {}
        }
    }
    count
}

/// Estimate the number of hydroxyl (–OH) groups in a SMILES string.
///
/// Counts aliphatic `O` atoms that are not carbonyl oxygens (`=O`) and not
/// ether oxygens (flanked by carbon on both sides).  Also recognises `[OH]`.
fn count_hydroxyls(smiles: &str) -> u32 {
    let chars: Vec<char> = smiles.chars().collect();
    let n = chars.len();
    let mut count = 0u32;
    let mut i = 0;

    while i < n {
        // Bracket atom: read until ']'
        if chars[i] == '[' {
            i += 1;
            let mut buf = String::new();
            while i < n && chars[i] != ']' {
                buf.push(chars[i]);
                i += 1;
            }
            i += 1; // skip ']'
            let sym = buf.trim_start_matches(|c: char| c.is_ascii_digit());
            if sym.starts_with("OH") {
                count += 1;
            }
            continue;
        }

        if chars[i] == 'O' {
            let prev = if i > 0 { chars[i - 1] } else { '\0' };
            let next = if i + 1 < n { chars[i + 1] } else { '\0' };

            // Skip carbonyl oxygen (=O)
            if prev == '=' {
                i += 1;
                continue;
            }

            // Skip ether oxygen: carbon-like on both sides
            let prev_is_c = matches!(prev, 'C' | 'c' | ')');
            let next_is_c = matches!(next, 'C' | 'c' | '(');
            if prev_is_c && next_is_c {
                i += 1;
                continue;
            }

            count += 1;
        }

        i += 1;
    }
    count
}

/// Return `true` when the SMILES contains a ring-closure digit outside brackets.
fn ring_present(smiles: &str) -> bool {
    let mut in_bracket = false;
    for ch in smiles.chars() {
        match ch {
            '[' => in_bracket = true,
            ']' => in_bracket = false,
            c if c.is_ascii_digit() && !in_bracket => return true,
            _ => {}
        }
    }
    false
}

/// Return `true` when a C=C aliphatic double bond is present.
fn cc_double_bond_present(smiles: &str) -> bool {
    // Direct C=C forms
    smiles.contains("C=C")
        || smiles.contains("c=c")
        || smiles.contains("C=c")
        || smiles.contains("c=C")
        // Branch form: C(=C)... e.g. methacrylic acid CC(=C)C(=O)O
        || smiles.contains("(=C)")
        || smiles.contains("(=c)")
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn fg(smiles: &str) -> Vec<FunctionalGroup> {
        detect_functional_groups(smiles)
    }

    fn has(smiles: &str, g: FunctionalGroup) -> bool {
        fg(smiles).contains(&g)
    }

    // ── Organic / inorganic ───────────────────────────────────────────────

    #[test]
    fn co2_is_inorganic() {
        assert_eq!(classify_organic("O=C=O"), OrganicInorganic::Inorganic);
    }

    #[test]
    fn water_is_inorganic() {
        assert_eq!(classify_organic("O"), OrganicInorganic::Inorganic);
    }

    #[test]
    fn ethanol_is_organic() {
        assert_eq!(classify_organic("CCO"), OrganicInorganic::Organic);
    }

    #[test]
    fn benzene_is_organic() {
        assert_eq!(classify_organic("c1ccccc1"), OrganicInorganic::Organic);
    }

    // ── Functional group detection ────────────────────────────────────────

    #[test]
    fn acetic_acid_detected() {
        // CC(=O)O — acetic acid (PubChem canonical)
        assert!(has("CC(=O)O", FunctionalGroup::CarboxylicAcid));
        assert!(!has("CC(=O)O", FunctionalGroup::Ester));
    }

    #[test]
    fn ethyl_acetate_detected_as_ester() {
        // CCOC(C)=O — ethyl acetate (PubChem canonical)
        assert!(has("CCOC(C)=O", FunctionalGroup::Ester));
        assert!(!has("CCOC(C)=O", FunctionalGroup::CarboxylicAcid));
    }

    #[test]
    fn phthalic_anhydride_detected() {
        // O=C1OC(=O)c2ccccc21
        let groups = fg("O=C1OC(=O)c2ccccc21");
        assert!(groups.contains(&FunctionalGroup::Anhydride));
        assert!(!groups.contains(&FunctionalGroup::Ester));
    }

    #[test]
    fn acetaldehyde_detected() {
        // CC=O
        assert!(has("CC=O", FunctionalGroup::Aldehyde));
        assert!(!has("CC=O", FunctionalGroup::Ketone));
    }

    /// Regression test: "CC=O" (acetaldehyde) must NOT be classified as Alcohol.
    /// The terminal "O" in "CC=O" was previously caught by the generic
    /// `smiles.ends_with("O")` check in the alcohol branch.
    #[test]
    fn acetaldehyde_not_classified_as_alcohol() {
        assert!(!has("CC=O", FunctionalGroup::Alcohol),
            "aldehyde SMILES 'CC=O' must not produce Alcohol group");
    }

    #[test]
    fn acetone_detected_as_ketone() {
        // CC(C)=O — PubChem canonical
        assert!(has("CC(C)=O", FunctionalGroup::Ketone));
        assert!(!has("CC(C)=O", FunctionalGroup::Aldehyde));
    }

    #[test]
    fn ethanol_detected_as_alcohol() {
        // CCO
        assert!(has("CCO", FunctionalGroup::Alcohol));
        assert!(!has("CCO", FunctionalGroup::Ether));
    }

    #[test]
    fn dimethyl_ether_detected() {
        // COC
        assert!(has("COC", FunctionalGroup::Ether));
        assert!(!has("COC", FunctionalGroup::Alcohol));
    }

    #[test]
    fn methylamine_detected() {
        // CN — methylamine
        assert!(has("CN", FunctionalGroup::Amine));
    }

    #[test]
    fn acetamide_detected() {
        // CC(N)=O — acetamide (PubChem canonical)
        assert!(has("CC(N)=O", FunctionalGroup::Amide));
        assert!(!has("CC(N)=O", FunctionalGroup::Ketone));
    }

    #[test]
    fn acetonitrile_detected() {
        // CC#N
        assert!(has("CC#N", FunctionalGroup::Nitrile));
    }

    #[test]
    fn chloromethane_detected() {
        // CCl
        assert!(has("CCl", FunctionalGroup::Halide));
    }

    #[test]
    fn ethylene_oxide_detected() {
        // C1CO1 — ethylene oxide (PubChem canonical)
        assert!(has("C1CO1", FunctionalGroup::Epoxide));
    }

    #[test]
    fn benzene_detected_as_aromatic() {
        assert!(has("c1ccccc1", FunctionalGroup::AromaticRing));
    }

    #[test]
    fn phenol_detected() {
        // Oc1ccccc1
        assert!(has("Oc1ccccc1", FunctionalGroup::Phenol));
    }

    #[test]
    fn nitrobenzene_detected() {
        // O=[N+]([O-])c1ccccc1
        assert!(has("O=[N+]([O-])c1ccccc1", FunctionalGroup::Nitro));
    }

    #[test]
    fn ethanesulfonic_acid_detected() {
        // CCS(=O)(=O)O
        assert!(has("CCS(=O)(=O)O", FunctionalGroup::SulphonicAcid));
    }

    #[test]
    fn dimethyl_sulfide_detected() {
        // CSC
        assert!(has("CSC", FunctionalGroup::Sulphide));
    }

    #[test]
    fn methanethiol_detected() {
        // C[SH]
        assert!(has("C[SH]", FunctionalGroup::Thiol));
    }

    #[test]
    fn isocyanate_detected() {
        // CN=C=O — methyl isocyanate
        assert!(has("CN=C=O", FunctionalGroup::Isocyanate));
    }

    #[test]
    fn trimethyl_phosphate_detected() {
        // COP(=O)(OC)OC
        assert!(has("COP(=O)(OC)OC", FunctionalGroup::Phosphate));
    }

    // ── StructuralFeatures ────────────────────────────────────────────────

    fn sf(smiles: &str) -> StructuralFeatures {
        detect_structural_features(smiles)
    }

    #[test]
    fn acetone_carbon_count_3() {
        // CC(C)=O — 3 carbons, no ring, no aromatic, no C=C
        let f = sf("CC(C)=O");
        assert_eq!(f.carbon_count, 3);
        assert!(!f.has_ring);
        assert!(!f.has_aromatic_ring);
        assert!(!f.has_cc_double_bond);
        assert_eq!(f.carbonyl_count, 1);
    }

    #[test]
    fn ethanol_hydroxyl_count_1() {
        // CCO — 2 carbons, 1 OH
        let f = sf("CCO");
        assert_eq!(f.carbon_count, 2);
        assert_eq!(f.hydroxyl_count, 1);
    }

    #[test]
    fn ethylene_glycol_hydroxyl_count_2() {
        // OCCO — 2 carbons, 2 OH
        let f = sf("OCCO");
        assert_eq!(f.carbon_count, 2);
        assert_eq!(f.hydroxyl_count, 2);
    }

    #[test]
    fn glycerol_hydroxyl_count_3() {
        // OCC(O)CO — 3 carbons, 3 OH
        let f = sf("OCC(O)CO");
        assert_eq!(f.carbon_count, 3);
        assert_eq!(f.hydroxyl_count, 3);
    }

    #[test]
    fn ether_oxygen_not_counted_as_oh() {
        // COC — dimethyl ether, 0 OH
        let f = sf("COC");
        assert_eq!(f.hydroxyl_count, 0);
    }

    #[test]
    fn acetic_acid_one_oh() {
        // CC(=O)O — acetic acid: 1 carbonyl + 1 acid OH
        let f = sf("CC(=O)O");
        assert_eq!(f.carbon_count, 2);
        assert_eq!(f.hydroxyl_count, 1);
        assert_eq!(f.carbonyl_count, 1);
    }

    #[test]
    fn acrylic_acid_has_cc_double_bond() {
        // C=CC(=O)O — acrylic acid: C=C present
        let f = sf("C=CC(=O)O");
        assert!(f.has_cc_double_bond);
        assert_eq!(f.carbon_count, 3);
    }

    #[test]
    fn methacrylic_acid_has_cc_double_bond() {
        // CC(=C)C(=O)O — methacrylic acid: branch C=C
        let f = sf("CC(=C)C(=O)O");
        assert!(f.has_cc_double_bond);
        assert_eq!(f.carbon_count, 4);
    }

    #[test]
    fn benzene_has_aromatic_ring() {
        let f = sf("c1ccccc1");
        assert!(f.has_ring);
        assert!(f.has_aromatic_ring);
        assert_eq!(f.carbon_count, 6);
    }

    #[test]
    fn cyclohexanone_is_ring_no_aromatic() {
        // O=C1CCCCC1 — cyclohexanone: 6C, ring, no aromatic
        let f = sf("O=C1CCCCC1");
        assert!(f.has_ring);
        assert!(!f.has_aromatic_ring);
        assert_eq!(f.carbon_count, 6);
    }

    #[test]
    fn chlorobenzene_has_halogen() {
        let f = sf("Clc1ccccc1");
        assert!(f.has_halogen);
        assert_eq!(f.carbon_count, 6);
    }

    #[test]
    fn methanol_carbon_count_1() {
        let f = sf("CO");
        assert_eq!(f.carbon_count, 1);
        assert_eq!(f.hydroxyl_count, 1);
    }
}
