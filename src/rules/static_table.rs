//! Compile-time embedded CAS → HS code mapping table.
//!
//! Rules are evaluated by [`find_best_rule`](crate::rules::matcher::find_best_rule)
//! in order. More-specific rules (non-`Any` shape, concentration range) are
//! preferred automatically by the matcher's specificity scoring.

use crate::rules::types::{HsRule, ShapePattern};

/// Static table of known CAS → HS 2022 mappings.
///
/// Target: ~100 common industrial chemicals for v0.1.
pub static HS_RULES: &[HsRule] = &[
    // ═══════════════════════════════════════════════════════════════
    // Chapter 28 — Inorganic chemicals
    // ═══════════════════════════════════════════════════════════════

    // ── Sodium hydroxide (caustic soda) 1310-73-2 ─────────────────
    HsRule {
        cas: "1310-73-2",
        shape: ShapePattern::Solid,
        purity_range: None,
        hs_code: "281511",
        heading_description: "Sodium hydroxide (caustic soda); solid",
        confidence: 0.97,
    },
    HsRule {
        cas: "1310-73-2",
        shape: ShapePattern::Powder,
        purity_range: None,
        hs_code: "281511",
        heading_description: "Sodium hydroxide (caustic soda); solid",
        confidence: 0.95,
    },
    HsRule {
        cas: "1310-73-2",
        shape: ShapePattern::Solution { concentration_range_pct: None },
        purity_range: None,
        hs_code: "281512",
        heading_description: "Sodium hydroxide (caustic soda); in aqueous solution (soda lye or liquid soda)",
        confidence: 0.97,
    },

    // ── Potassium hydroxide (caustic potash) 1310-58-3 ───────────
    HsRule {
        cas: "1310-58-3",
        shape: ShapePattern::Solid,
        purity_range: None,
        hs_code: "281520",
        heading_description: "Potassium hydroxide (caustic potash)",
        confidence: 0.97,
    },
    HsRule {
        cas: "1310-58-3",
        shape: ShapePattern::Solution { concentration_range_pct: None },
        purity_range: None,
        hs_code: "281520",
        heading_description: "Potassium hydroxide (caustic potash)",
        confidence: 0.95,
    },

    // ── Sulphuric acid 7664-93-9 ──────────────────────────────────
    HsRule {
        cas: "7664-93-9",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "280700",
        heading_description: "Sulphuric acid; oleum",
        confidence: 0.97,
    },

    // ── Hydrochloric acid / hydrogen chloride 7647-01-0 ──────────
    HsRule {
        cas: "7647-01-0",
        shape: ShapePattern::Gas,
        purity_range: None,
        hs_code: "280610",
        heading_description: "Hydrogen chloride (hydrochloric acid)",
        confidence: 0.97,
    },
    HsRule {
        cas: "7647-01-0",
        shape: ShapePattern::Solution { concentration_range_pct: None },
        purity_range: None,
        hs_code: "280610",
        heading_description: "Hydrogen chloride (hydrochloric acid)",
        confidence: 0.95,
    },
    HsRule {
        cas: "7647-01-0",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "280610",
        heading_description: "Hydrogen chloride (hydrochloric acid)",
        confidence: 0.90,
    },

    // ── Nitric acid 7697-37-2 ─────────────────────────────────────
    // Fuming nitric acid (concentration ≥98%)
    HsRule {
        cas: "7697-37-2",
        shape: ShapePattern::Solution { concentration_range_pct: Some(98.0..=100.0) },
        purity_range: None,
        hs_code: "280810",
        heading_description: "Nitric acid; fuming nitric acid",
        confidence: 0.90,
    },
    // Standard nitric acid (<98%)
    HsRule {
        cas: "7697-37-2",
        shape: ShapePattern::Solution { concentration_range_pct: Some(0.0..=97.99) },
        purity_range: None,
        hs_code: "280890",
        heading_description: "Nitric acid",
        confidence: 0.90,
    },
    HsRule {
        cas: "7697-37-2",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "280800",
        heading_description: "Nitric acid; sulphonitric acids",
        confidence: 0.75, // lower: concentration unknown
    },

    // ── Phosphoric acid 7664-38-2 ─────────────────────────────────
    HsRule {
        cas: "7664-38-2",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "280920",
        heading_description: "Phosphoric acid and polyphosphoric acids",
        confidence: 0.97,
    },

    // ── Hydrofluoric acid / hydrogen fluoride 7664-39-3 ──────────
    HsRule {
        cas: "7664-39-3",
        shape: ShapePattern::Gas,
        purity_range: None,
        hs_code: "281111",
        heading_description: "Hydrogen fluoride (hydrofluoric acid)",
        confidence: 0.97,
    },
    HsRule {
        cas: "7664-39-3",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "281111",
        heading_description: "Hydrogen fluoride (hydrofluoric acid)",
        confidence: 0.90,
    },

    // ── Ammonia 7664-41-7 ─────────────────────────────────────────
    HsRule {
        cas: "7664-41-7",
        shape: ShapePattern::Gas,
        purity_range: None,
        hs_code: "281410",
        heading_description: "Anhydrous ammonia",
        confidence: 0.97,
    },
    HsRule {
        cas: "7664-41-7",
        shape: ShapePattern::Solution { concentration_range_pct: None },
        purity_range: None,
        hs_code: "281420",
        heading_description: "Ammonia in aqueous solution",
        confidence: 0.97,
    },
    HsRule {
        cas: "7664-41-7",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "281400",
        heading_description: "Ammonia, anhydrous or in aqueous solution",
        confidence: 0.75,
    },

    // ── Chlorine 7782-50-5 ────────────────────────────────────────
    HsRule {
        cas: "7782-50-5",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "280110",
        heading_description: "Chlorine",
        confidence: 0.97,
    },

    // ── Bromine 7726-95-6 ─────────────────────────────────────────
    HsRule {
        cas: "7726-95-6",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "280130",
        heading_description: "Bromine",
        confidence: 0.97,
    },

    // ── Sodium chloride (salt) 7647-14-5 ─────────────────────────
    HsRule {
        cas: "7647-14-5",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "250100",
        heading_description: "Salt (including table salt and denatured salt)",
        confidence: 0.90, // Chapter 25, not 28
    },

    // ── Sodium carbonate 497-19-8 ────────────────────────────────
    HsRule {
        cas: "497-19-8",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "283620",
        heading_description: "Sodium carbonate",
        confidence: 0.97,
    },

    // ── Sodium bicarbonate 144-55-8 ───────────────────────────────
    HsRule {
        cas: "144-55-8",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "283630",
        heading_description: "Sodium bicarbonate (sodium hydrogen carbonate)",
        confidence: 0.97,
    },

    // ── Calcium carbonate 471-34-1 ────────────────────────────────
    HsRule {
        cas: "471-34-1",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "283650",
        heading_description: "Calcium carbonate",
        confidence: 0.90, // also appears in Ch.25 as limestone
    },

    // ── Calcium hydroxide 1305-62-0 ───────────────────────────────
    HsRule {
        cas: "1305-62-0",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "282520",
        heading_description: "Calcium hydroxide",
        confidence: 0.97,
    },

    // ── Calcium chloride 10043-52-4 ───────────────────────────────
    HsRule {
        cas: "10043-52-4",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "282720",
        heading_description: "Calcium chloride",
        confidence: 0.97,
    },

    // ── Aluminium oxide 1344-28-1 ─────────────────────────────────
    HsRule {
        cas: "1344-28-1",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "281820",
        heading_description: "Aluminium oxide (other than artificial corundum)",
        confidence: 0.90, // artificial corundum → 281810
    },

    // ── Titanium dioxide 13463-67-7 ───────────────────────────────
    HsRule {
        cas: "13463-67-7",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "282300",
        heading_description: "Titanium oxides",
        confidence: 0.97,
    },

    // ── Iron(III) chloride 7705-08-0 ──────────────────────────────
    HsRule {
        cas: "7705-08-0",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "282739",
        heading_description: "Chlorides of other metals — iron(III) chloride",
        confidence: 0.95,
    },

    // ── Copper(II) sulphate 7758-98-7 ────────────────────────────
    HsRule {
        cas: "7758-98-7",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "283325",
        heading_description: "Copper sulphates",
        confidence: 0.97,
    },

    // ── Silica (precipitated/fumed) 7631-86-9 ────────────────────
    HsRule {
        cas: "7631-86-9",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "281122",
        heading_description: "Silicon dioxide (synthetic)",
        confidence: 0.93,
    },

    // ── Sodium silicate 1344-09-8 ─────────────────────────────────
    HsRule {
        cas: "1344-09-8",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "283911",
        heading_description: "Sodium silicates — sodium metasilicate",
        confidence: 0.88,
    },

    // ── Hydrogen peroxide 7722-84-1 ───────────────────────────────
    HsRule {
        cas: "7722-84-1",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "284700",
        heading_description: "Hydrogen peroxide",
        confidence: 0.97,
    },

    // ── Sodium hypochlorite 7681-52-9 ────────────────────────────
    HsRule {
        cas: "7681-52-9",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "282810",
        heading_description: "Hypochlorites — sodium hypochlorite",
        confidence: 0.97,
    },

    // ── Sodium sulphate 7757-82-6 ─────────────────────────────────
    HsRule {
        cas: "7757-82-6",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "283311",
        heading_description: "Disodium sulphate",
        confidence: 0.97,
    },

    // ── Potassium chloride 7447-40-7 ─────────────────────────────
    HsRule {
        cas: "7447-40-7",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "283110",
        heading_description: "Potassium chloride",
        confidence: 0.93, // also Ch.31 as fertiliser depending on grade
    },

    // ── Ammonium nitrate 6484-52-2 ───────────────────────────────
    HsRule {
        cas: "6484-52-2",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "310230",
        heading_description: "Ammonium nitrate (Ch.31 fertiliser heading)",
        confidence: 0.85, // can be Ch.28 in some contexts
    },

    // ═══════════════════════════════════════════════════════════════
    // Chapter 29 — Organic chemicals
    // ═══════════════════════════════════════════════════════════════

    // ── Ethanol 64-17-5 ───────────────────────────────────────────
    HsRule {
        cas: "64-17-5",
        shape: ShapePattern::Any,
        purity_range: Some(95.0..=100.0), // undenatured ≥95%
        hs_code: "220710",
        heading_description: "Undenatured ethyl alcohol of an alcoholic strength ≥80%",
        confidence: 0.85,
    },
    HsRule {
        cas: "64-17-5",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "290511",
        heading_description: "Ethanol (methanol)",
        confidence: 0.70, // varies by denaturation / concentration
    },

    // ── Methanol 67-56-1 ─────────────────────────────────────────
    HsRule {
        cas: "67-56-1",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "290511",
        heading_description: "Methanol (methyl alcohol)",
        confidence: 0.97,
    },

    // ── Acetone 67-64-1 ───────────────────────────────────────────
    HsRule {
        cas: "67-64-1",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "291411",
        heading_description: "Acetone",
        confidence: 0.97,
    },

    // ── Acetic acid 64-19-7 ───────────────────────────────────────
    HsRule {
        cas: "64-19-7",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "291521",
        heading_description: "Acetic acid",
        confidence: 0.97,
    },

    // ── Toluene 108-88-3 ─────────────────────────────────────────
    HsRule {
        cas: "108-88-3",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "290230",
        heading_description: "Toluene",
        confidence: 0.97,
    },

    // ── Benzene 71-43-2 ───────────────────────────────────────────
    HsRule {
        cas: "71-43-2",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "290220",
        heading_description: "Benzene",
        confidence: 0.97,
    },

    // ── Xylene (mixed isomers) 1330-20-7 ─────────────────────────
    HsRule {
        cas: "1330-20-7",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "290244",
        heading_description: "Mixed xylene isomers",
        confidence: 0.93,
    },

    // ── Isopropanol 67-63-0 ───────────────────────────────────────
    HsRule {
        cas: "67-63-0",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "290512",
        heading_description: "Propan-2-ol (isopropyl alcohol)",
        confidence: 0.97,
    },

    // ── Ethyl acetate 141-78-6 ────────────────────────────────────
    HsRule {
        cas: "141-78-6",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "291523",
        heading_description: "Ethyl acetate",
        confidence: 0.97,
    },

    // ── Formaldehyde 50-00-0 ──────────────────────────────────────
    HsRule {
        cas: "50-00-0",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "291211",
        heading_description: "Methanal (formaldehyde)",
        confidence: 0.97,
    },

    // ── Formic acid 64-18-6 ───────────────────────────────────────
    HsRule {
        cas: "64-18-6",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "291511",
        heading_description: "Formic acid",
        confidence: 0.97,
    },

    // ── Citric acid 77-92-9 ───────────────────────────────────────
    HsRule {
        cas: "77-92-9",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "291814",
        heading_description: "Citric acid",
        confidence: 0.97,
    },

    // ── Urea 57-13-6 ─────────────────────────────────────────────
    HsRule {
        cas: "57-13-6",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "310210",
        heading_description: "Urea (Ch.31 as fertiliser)",
        confidence: 0.85,
    },

    // ── Aniline 62-53-3 ───────────────────────────────────────────
    HsRule {
        cas: "62-53-3",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "292141",
        heading_description: "Aniline",
        confidence: 0.97,
    },

    // ── Phenol 108-95-2 ───────────────────────────────────────────
    HsRule {
        cas: "108-95-2",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "290711",
        heading_description: "Phenol",
        confidence: 0.97,
    },

    // ═══════════════════════════════════════════════════════════════
    // Metals (Chapters 72–81)
    // ═══════════════════════════════════════════════════════════════

    // ── Aluminium 7429-90-5 ───────────────────────────────────────
    HsRule {
        cas: "7429-90-5",
        shape: ShapePattern::Ingot,
        purity_range: Some(99.0..=100.0),
        hs_code: "760110",
        heading_description: "Aluminium, not alloyed — ingots, billets",
        confidence: 0.93,
    },
    HsRule {
        cas: "7429-90-5",
        shape: ShapePattern::Powder,
        purity_range: None,
        hs_code: "760310",
        heading_description: "Aluminium powders of non-lamellar structure",
        confidence: 0.92,
    },
    HsRule {
        cas: "7429-90-5",
        shape: ShapePattern::Foil,
        purity_range: None,
        hs_code: "760711",
        heading_description: "Aluminium foil (not backed), rolled but not further worked",
        confidence: 0.90,
    },
    HsRule {
        cas: "7429-90-5",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "760110",
        heading_description: "Aluminium — unwrought",
        confidence: 0.70,
    },

    // ── Copper 7440-50-8 ─────────────────────────────────────────
    HsRule {
        cas: "7440-50-8",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "740311",
        heading_description: "Refined copper — cathodes and sections of cathodes",
        confidence: 0.80,
    },

    // ── Iron / steel (pure iron) 7439-89-6 ───────────────────────
    HsRule {
        cas: "7439-89-6",
        shape: ShapePattern::Powder,
        purity_range: None,
        hs_code: "720310",
        heading_description: "Ferrous products obtained by direct reduction of iron ore",
        confidence: 0.70, // many iron/steel codes — low confidence
    },

    // ── Zinc 7440-66-6 ────────────────────────────────────────────
    HsRule {
        cas: "7440-66-6",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "790111",
        heading_description: "Zinc, not alloyed — ≥99.99% pure",
        confidence: 0.82,
    },

    // ═══════════════════════════════════════════════════════════════
    // Chapter 28 — Inorganic chemicals (continued)
    // ═══════════════════════════════════════════════════════════════

    // ── Zinc oxide 1314-13-2 ──────────────────────────────────────
    HsRule {
        cas: "1314-13-2",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "281700",
        heading_description: "Zinc oxide; zinc peroxide",
        confidence: 0.97,
    },

    // ── Iron(III) oxide (haematite) 1309-37-1 ────────────────────
    HsRule {
        cas: "1309-37-1",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "282110",
        heading_description: "Iron oxides and hydroxides",
        confidence: 0.93,
    },

    // ── Manganese dioxide 1313-13-9 ───────────────────────────────
    HsRule {
        cas: "1313-13-9",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "282010",
        heading_description: "Manganese dioxide",
        confidence: 0.97,
    },

    // ── Potassium permanganate 7722-64-7 ──────────────────────────
    HsRule {
        cas: "7722-64-7",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "284130",
        heading_description: "Potassium permanganate",
        confidence: 0.97,
    },

    // ── Sodium sulphide 1313-82-2 ─────────────────────────────────
    HsRule {
        cas: "1313-82-2",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "283010",
        heading_description: "Sodium sulphides",
        confidence: 0.97,
    },

    // ── Zinc sulphate 7733-02-0 ───────────────────────────────────
    HsRule {
        cas: "7733-02-0",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "283329",
        heading_description: "Sulphates — zinc sulphate",
        confidence: 0.95,
    },

    // ── Aluminium sulphate 10043-01-3 ────────────────────────────
    HsRule {
        cas: "10043-01-3",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "283322",
        heading_description: "Aluminium sulphate",
        confidence: 0.97,
    },

    // ── Boric acid 10043-35-3 ─────────────────────────────────────
    HsRule {
        cas: "10043-35-3",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "281000",
        heading_description: "Oxides of boron; boric acids",
        confidence: 0.97,
    },

    // ── Sodium nitrate 7631-99-4 ─────────────────────────────────
    HsRule {
        cas: "7631-99-4",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "283421",
        heading_description: "Sodium nitrate",
        confidence: 0.97,
    },

    // ── Potassium nitrate 7757-79-1 ──────────────────────────────
    HsRule {
        cas: "7757-79-1",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "283410",
        heading_description: "Potassium nitrate",
        confidence: 0.97,
    },

    // ── Potassium carbonate 584-08-7 ─────────────────────────────
    HsRule {
        cas: "584-08-7",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "283640",
        heading_description: "Potassium carbonates",
        confidence: 0.97,
    },

    // ── Zinc chloride 7646-85-7 ──────────────────────────────────
    HsRule {
        cas: "7646-85-7",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "282741",
        heading_description: "Zinc chloride",
        confidence: 0.97,
    },

    // ── Chromium trioxide 1333-82-0 ───────────────────────────────
    HsRule {
        cas: "1333-82-0",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "281910",
        heading_description: "Chromium trioxide",
        confidence: 0.97,
    },

    // ── Silver nitrate 7761-88-8 ──────────────────────────────────
    HsRule {
        cas: "7761-88-8",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "284321",
        heading_description: "Silver nitrate",
        confidence: 0.97,
    },

    // ── Ferrous sulphate (iron(II) sulphate) 7720-78-7 ───────────
    HsRule {
        cas: "7720-78-7",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "283329",
        heading_description: "Sulphates — ferrous sulphate (iron(II) sulphate)",
        confidence: 0.93,
    },

    // ─══════════════════════════════════════════════════════════════
    // Chapter 29 — Organic chemicals (continued)
    // ═══════════════════════════════════════════════════════════════

    // ── n-Hexane 110-54-3 ─────────────────────────────────────────
    HsRule {
        cas: "110-54-3",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "290110",
        heading_description: "Saturated acyclic hydrocarbons — hexanes",
        confidence: 0.93,
    },

    // ── Cyclohexane 110-82-7 ──────────────────────────────────────
    HsRule {
        cas: "110-82-7",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "290211",
        heading_description: "Cyclohexane",
        confidence: 0.97,
    },

    // ── 1-Butanol (n-butyl alcohol) 71-36-3 ──────────────────────
    HsRule {
        cas: "71-36-3",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "290513",
        heading_description: "Butan-1-ol (n-butyl alcohol)",
        confidence: 0.97,
    },

    // ── Ethylene glycol 107-21-1 ─────────────────────────────────
    HsRule {
        cas: "107-21-1",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "290531",
        heading_description: "Ethylene glycol (ethanediol)",
        confidence: 0.97,
    },

    // ── Propylene glycol (1,2-propanediol) 57-55-6 ───────────────
    HsRule {
        cas: "57-55-6",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "290532",
        heading_description: "Propylene glycol (1,2-propanediol)",
        confidence: 0.97,
    },

    // ── 1-Propanol (propyl alcohol) 71-23-8 ──────────────────────
    HsRule {
        cas: "71-23-8",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "290512",
        heading_description: "Propan-1-ol (propyl alcohol)",
        confidence: 0.97,
    },

    // ── Diethyl ether 60-29-7 ────────────────────────────────────
    HsRule {
        cas: "60-29-7",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "290911",
        heading_description: "Diethyl ether",
        confidence: 0.97,
    },

    // ── Dichloromethane (methylene chloride) 75-09-2 ─────────────
    HsRule {
        cas: "75-09-2",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "290312",
        heading_description: "Dichloromethane (methylene chloride)",
        confidence: 0.97,
    },

    // ── Chloroform (trichloromethane) 67-66-3 ────────────────────
    HsRule {
        cas: "67-66-3",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "290313",
        heading_description: "Chloroform (trichloromethane)",
        confidence: 0.97,
    },

    // ── Carbon tetrachloride 56-23-5 ─────────────────────────────
    HsRule {
        cas: "56-23-5",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "290314",
        heading_description: "Carbon tetrachloride",
        confidence: 0.97,
    },

    // ── Oxalic acid 144-62-7 ─────────────────────────────────────
    HsRule {
        cas: "144-62-7",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "291711",
        heading_description: "Oxalic acid, its salts and esters",
        confidence: 0.97,
    },

    // ── Lactic acid 50-21-5 ───────────────────────────────────────
    HsRule {
        cas: "50-21-5",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "291811",
        heading_description: "Lactic acid, its salts and esters",
        confidence: 0.97,
    },

    // ── Benzoic acid 65-85-0 ─────────────────────────────────────
    HsRule {
        cas: "65-85-0",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "291631",
        heading_description: "Benzoic acid, its salts and esters",
        confidence: 0.97,
    },

    // ── Phthalic anhydride 85-44-9 ───────────────────────────────
    HsRule {
        cas: "85-44-9",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "291735",
        heading_description: "Phthalic anhydride",
        confidence: 0.97,
    },

    // ── Ethylene oxide 75-21-8 ───────────────────────────────────
    HsRule {
        cas: "75-21-8",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "291010",
        heading_description: "Ethylene oxide",
        confidence: 0.97,
    },

    // ── Dimethyl sulphoxide (DMSO) 67-68-5 ───────────────────────
    HsRule {
        cas: "67-68-5",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "293090",
        heading_description: "Other organo-sulphur compounds — DMSO",
        confidence: 0.93,
    },

    // ── N,N-Dimethylformamide (DMF) 68-12-2 ─────────────────────
    HsRule {
        cas: "68-12-2",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "292419",
        heading_description: "Other acyclic amides — DMF",
        confidence: 0.95,
    },

    // ── Sodium acetate 127-09-3 ───────────────────────────────────
    HsRule {
        cas: "127-09-3",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "291529",
        heading_description: "Other salts of acetic acid — sodium acetate",
        confidence: 0.95,
    },

    // ═══════════════════════════════════════════════════════════════
    // Metals (continued — Chapters 75, 78, 80, 71)
    // ═══════════════════════════════════════════════════════════════

    // ── Nickel 7440-02-0 ─────────────────────────────────────────
    HsRule {
        cas: "7440-02-0",
        shape: ShapePattern::Ingot,
        purity_range: Some(99.0..=100.0),
        hs_code: "750110",
        heading_description: "Nickel, not alloyed — ingots etc.",
        confidence: 0.90,
    },
    HsRule {
        cas: "7440-02-0",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "750110",
        heading_description: "Nickel, not alloyed — unwrought",
        confidence: 0.75,
    },

    // ── Lead 7439-92-1 ────────────────────────────────────────────
    HsRule {
        cas: "7439-92-1",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "780110",
        heading_description: "Refined lead — unwrought",
        confidence: 0.78,
    },

    // ── Tin 7440-31-5 ─────────────────────────────────────────────
    HsRule {
        cas: "7440-31-5",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "800110",
        heading_description: "Tin, not alloyed — unwrought",
        confidence: 0.80,
    },

    // ── Silver 7440-22-4 ─────────────────────────────────────────
    HsRule {
        cas: "7440-22-4",
        shape: ShapePattern::Any,
        purity_range: None,
        hs_code: "710691",
        heading_description: "Silver — unwrought",
        confidence: 0.82,
    },
];
