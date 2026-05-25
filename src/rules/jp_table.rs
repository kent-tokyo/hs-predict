//! Japan Customs 統計品目番号 (statistical item code) lookup table.
//!
//! Japanese tariff codes are 9-digit numbers that extend the 6-digit HS code
//! with Japan-specific sub-classifications (e.g. `"281511000"`).
//!
//! Source: 実行関税率表 (Japan Customs Tariff Schedule)
//! Valid from: **2026-04-01** (2026年4月改正)
//! Reference: <https://www.customs.go.jp/tariff/2026_04_01/index.htm>
//!
//! **Note**: The tariff schedule is revised annually. Verify codes against the
//! current schedule before using in customs declarations.

/// A Japan tariff entry mapping an HS heading to a statistical item code.
#[derive(Debug)]
pub struct JpRule {
    /// Six-digit HS 2022 code (e.g. `"281511"`).
    pub hs_code: &'static str,

    /// Nine-digit Japan statistical item code (e.g. `"281511000"`).
    ///
    /// For most basic chemicals the last three digits are `"000"`, indicating
    /// no Japan-specific sub-classification within that HS subheading.
    pub jp_code: &'static str,

    /// Short description in Japanese.
    pub jp_description: &'static str,

    /// Applicable tariff rate (informational, e.g. `"free"` or `"6.5%"`).
    /// `None` when the rate is complex (depends on end-use, origin, etc.).
    pub tariff_rate: Option<&'static str>,
}

/// Tariff schedule year used for [`JP_RULES`].
pub const JP_TARIFF_YEAR: u16 = 2026;

/// Static Japan tariff code table (HS 2022 → 統計品目番号 2026).
///
/// Ordered by HS code for readability.
pub static JP_RULES: &[JpRule] = &[
    // ═══════════════════════════════════════════════
    // Chapter 25
    // ═══════════════════════════════════════════════
    JpRule {
        hs_code: "250100",
        jp_code: "250100000",
        jp_description: "塩化ナトリウム（食塩等）",
        tariff_rate: Some("free"),
    },

    // ═══════════════════════════════════════════════
    // Chapter 28 — 無機化学品
    // ═══════════════════════════════════════════════
    JpRule {
        hs_code: "280110",
        jp_code: "280110000",
        jp_description: "塩素",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "280130",
        jp_code: "280130000",
        jp_description: "臭素",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "280610",
        jp_code: "280610000",
        jp_description: "塩化水素（塩酸）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "280700",
        jp_code: "280700000",
        jp_description: "硫酸；発煙硫酸",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "280800",
        jp_code: "280800000",
        jp_description: "硝酸；混酸",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "280810",
        jp_code: "280810000",
        jp_description: "発煙硝酸",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "280890",
        jp_code: "280890000",
        jp_description: "硝酸（その他）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "280920",
        jp_code: "280920000",
        jp_description: "リン酸及びポリリン酸",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "281000",
        jp_code: "281000000",
        jp_description: "ホウ素の酸化物；ホウ酸",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "281111",
        jp_code: "281111000",
        jp_description: "フッ化水素（フッ酸）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "281122",
        jp_code: "281122000",
        jp_description: "二酸化ケイ素（合成）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "281400",
        jp_code: "281400000",
        jp_description: "アンモニア（無水又は水溶液）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "281410",
        jp_code: "281410000",
        jp_description: "アンモニア（無水）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "281420",
        jp_code: "281420000",
        jp_description: "アンモニア水",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "281511",
        jp_code: "281511000",
        jp_description: "水酸化ナトリウム（固体）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "281512",
        jp_code: "281512000",
        jp_description: "水酸化ナトリウム水溶液（苛性ソーダ液）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "281520",
        jp_code: "281520000",
        jp_description: "水酸化カリウム（苛性カリ）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "281600",
        jp_code: "281600000",
        jp_description: "酸化亜鉛；過酸化亜鉛",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "281700",
        jp_code: "281700000",
        jp_description: "酸化亜鉛；過酸化亜鉛",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "281820",
        jp_code: "281820000",
        jp_description: "酸化アルミニウム（人造コランダム以外）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "281910",
        jp_code: "281910000",
        jp_description: "三酸化クロム",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "282010",
        jp_code: "282010000",
        jp_description: "二酸化マンガン",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "282110",
        jp_code: "282110000",
        jp_description: "鉄の酸化物及び水酸化物",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "282200",
        jp_code: "282200000",
        jp_description: "鉄のコバルト酸化物；コバルト酸化物",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "282300",
        jp_code: "282300000",
        jp_description: "チタンの酸化物",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "282520",
        jp_code: "282520000",
        jp_description: "水酸化カルシウム",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "282720",
        jp_code: "282720000",
        jp_description: "塩化カルシウム",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "282739",
        jp_code: "282739000",
        jp_description: "その他の金属の塩化物（塩化第二鉄等）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "282741",
        jp_code: "282741000",
        jp_description: "塩化亜鉛",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "282810",
        jp_code: "282810000",
        jp_description: "次亜塩素酸ナトリウム",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "283010",
        jp_code: "283010000",
        jp_description: "硫化ナトリウム",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "283110",
        jp_code: "283110000",
        jp_description: "塩化カリウム",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "283311",
        jp_code: "283311000",
        jp_description: "硫酸二ナトリウム",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "283322",
        jp_code: "283322000",
        jp_description: "硫酸アルミニウム",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "283325",
        jp_code: "283325000",
        jp_description: "硫酸銅",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "283329",
        jp_code: "283329000",
        jp_description: "その他の硫酸塩（硫酸亜鉛等）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "283410",
        jp_code: "283410000",
        jp_description: "亜硝酸塩（亜硝酸ナトリウム等）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "283421",
        jp_code: "283421000",
        jp_description: "硝酸カリウム",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "283530",
        jp_code: "283530000",
        jp_description: "リン酸ナトリウム（三リン酸ナトリウム等）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "283531",
        jp_code: "283531000",
        jp_description: "三リン酸ナトリウム（トリポリリン酸ナトリウム）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "283620",
        jp_code: "283620000",
        jp_description: "炭酸ナトリウム",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "283630",
        jp_code: "283630000",
        jp_description: "炭酸水素ナトリウム",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "283640",
        jp_code: "283640000",
        jp_description: "炭酸カリウム",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "283650",
        jp_code: "283650000",
        jp_description: "炭酸カルシウム",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "283911",
        jp_code: "283911000",
        jp_description: "ケイ酸ナトリウム（メタケイ酸ナトリウム等）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "284130",
        jp_code: "284130000",
        jp_description: "重クロム酸ナトリウム",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "284150",
        jp_code: "284150000",
        jp_description: "その他のクロム酸塩及び重クロム酸塩（重クロム酸カリウム等）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "284161",
        jp_code: "284161000",
        jp_description: "過マンガン酸カリウム",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "284321",
        jp_code: "284321000",
        jp_description: "硝酸銀",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "284700",
        jp_code: "284700000",
        jp_description: "過酸化水素",
        tariff_rate: Some("free"),
    },

    // ═══════════════════════════════════════════════
    // Chapter 29 — 有機化学品
    // ═══════════════════════════════════════════════
    JpRule {
        hs_code: "290110",
        jp_code: "290110000",
        jp_description: "飽和非環式炭化水素（ヘキサン等）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "290211",
        jp_code: "290211000",
        jp_description: "シクロヘキサン",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "290220",
        jp_code: "290220000",
        jp_description: "ベンゼン",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "290230",
        jp_code: "290230000",
        jp_description: "トルエン",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "290244",
        jp_code: "290244000",
        jp_description: "混合キシレン異性体",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "290312",
        jp_code: "290312000",
        jp_description: "ジクロロメタン（塩化メチレン）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "290313",
        jp_code: "290313000",
        jp_description: "クロロホルム（トリクロロメタン）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "290314",
        jp_code: "290314000",
        jp_description: "四塩化炭素",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "290511",
        jp_code: "290511000",
        jp_description: "メタノール（メチルアルコール）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "290512",
        jp_code: "290512000",
        jp_description: "プロパン-1-オール（プロピルアルコール）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "290513",
        jp_code: "290513000",
        jp_description: "ブタン-1-オール（n-ブチルアルコール）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "290531",
        jp_code: "290531000",
        jp_description: "エチレングリコール（エタンジオール）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "290532",
        jp_code: "290532000",
        jp_description: "プロピレングリコール（1,2-プロパンジオール）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "290911",
        jp_code: "290911000",
        jp_description: "ジエチルエーテル",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291010",
        jp_code: "291010000",
        jp_description: "エチレンオキサイド（酸化エチレン）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291211",
        jp_code: "291211000",
        jp_description: "ホルムアルデヒド（ホルマリン）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291411",
        jp_code: "291411000",
        jp_description: "アセトン",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291511",
        jp_code: "291511000",
        jp_description: "ギ酸",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291521",
        jp_code: "291521000",
        jp_description: "酢酸",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291523",
        jp_code: "291523000",
        jp_description: "酢酸エチル",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291529",
        jp_code: "291529000",
        jp_description: "酢酸塩（酢酸ナトリウム等）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291631",
        jp_code: "291631000",
        jp_description: "安息香酸、その塩及びエステル",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291711",
        jp_code: "291711000",
        jp_description: "シュウ酸、その塩及びエステル",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291735",
        jp_code: "291735000",
        jp_description: "無水フタル酸",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291811",
        jp_code: "291811000",
        jp_description: "乳酸、その塩及びエステル",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291814",
        jp_code: "291814000",
        jp_description: "クエン酸",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "292141",
        jp_code: "292141000",
        jp_description: "アニリン",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "292419",
        jp_code: "292419000",
        jp_description: "その他の非環式アミド（DMF等）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "290711",
        jp_code: "290711000",
        jp_description: "フェノール",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "293090",
        jp_code: "293090000",
        jp_description: "その他の有機硫黄化合物（DMSO等）",
        tariff_rate: Some("free"),
    },

    // ═══════════════════════════════════════════════
    // Chapter 31 — 肥料
    // ═══════════════════════════════════════════════
    JpRule {
        hs_code: "310210",
        jp_code: "310210000",
        jp_description: "尿素",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "310230",
        jp_code: "310230000",
        jp_description: "硝酸アンモニウム",
        tariff_rate: Some("free"),
    },

    // ═══════════════════════════════════════════════
    // Chapter 72–81 — 金属
    // ═══════════════════════════════════════════════
    JpRule {
        hs_code: "720310",
        jp_code: "720310000",
        jp_description: "直接還元鉄鉱石で得た鉄製品",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "740311",
        jp_code: "740311000",
        jp_description: "精製銅（カソード等）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "750110",
        jp_code: "750110000",
        jp_description: "ニッケル（未加工、合金でないもの）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "760110",
        jp_code: "760110000",
        jp_description: "アルミニウム（未加工、合金でないもの）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "760310",
        jp_code: "760310000",
        jp_description: "アルミニウム粉末（非板状構造）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "760711",
        jp_code: "760711000",
        jp_description: "アルミニウム箔（裏打ちなし）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "780110",
        jp_code: "780110000",
        jp_description: "精製鉛（未加工）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "790111",
        jp_code: "790111000",
        jp_description: "亜鉛（未加工、純度≥99.99%）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "800110",
        jp_code: "800110000",
        jp_description: "スズ（未加工、合金でないもの）",
        tariff_rate: Some("free"),
    },

    // ═══════════════════════════════════════════════
    // Chapter 71 — 貴金属
    // ═══════════════════════════════════════════════
    JpRule {
        hs_code: "710691",
        jp_code: "710691000",
        jp_description: "銀（未加工）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "710812",
        jp_code: "710812000",
        jp_description: "金（非貨幣用、未加工）",
        tariff_rate: Some("free"),
    },

    // ═══════════════════════════════════════════════
    // Chapter 22
    // ═══════════════════════════════════════════════
    JpRule {
        hs_code: "220710",
        jp_code: "220710000",
        jp_description: "変性していないエチルアルコール（アルコール分≥80%）",
        tariff_rate: None, // complex rate
    },

    // ═══════════════════════════════════════════════
    // Chapter 28 — v0.5 additions
    // ═══════════════════════════════════════════════
    JpRule {
        hs_code: "280410",
        jp_code: "280410000",
        jp_description: "水素",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "280420",
        jp_code: "280420000",
        jp_description: "窒素",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "280440",
        jp_code: "280440000",
        jp_description: "酸素",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "281121",
        jp_code: "281121000",
        jp_description: "二酸化炭素",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "281129",
        jp_code: "281129000",
        jp_description: "非金属の無機酸素化合物（二酸化硫黄等）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "281830",
        jp_code: "281830000",
        jp_description: "水酸化アルミニウム",
        tariff_rate: Some("free"),
    },
    // NOTE: 281921 and 281929 were removed — they are not valid HS 2022 subheadings.
    // Heading 28.19 (chromium oxides/hydroxides) only has subheadings .10 and .90.
    // Dichromates belong under heading 28.41: see 284130 and 284150 below.
    JpRule {
        hs_code: "282611",
        jp_code: "282611000",
        jp_description: "フッ化物（アンモニウム又はナトリウム）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "282710",
        jp_code: "282710000",
        jp_description: "塩化アンモニウム",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "282731",
        jp_code: "282731000",
        jp_description: "塩化マグネシウム",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "282732",
        jp_code: "282732000",
        jp_description: "塩化アルミニウム",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "282733",
        jp_code: "282733000",
        jp_description: "塩化鉄",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "282735",
        jp_code: "282735000",
        jp_description: "塩化ニッケル",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "282746",
        jp_code: "282746000",
        jp_description: "塩化銅",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "283210",
        jp_code: "283210000",
        jp_description: "亜硫酸ナトリウム（メタ重亜硫酸ナトリウム等）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "283220",
        jp_code: "283220000",
        jp_description: "チオ硫酸ナトリウム",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "283319",
        jp_code: "283319000",
        jp_description: "アルカリ金属の硫酸塩（硫酸カリウム等）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "283321",
        jp_code: "283321000",
        jp_description: "硫酸マグネシウム",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "283429",
        jp_code: "283429000",
        jp_description: "その他の硝酸塩（硝酸カルシウム、硝酸銅等）",
        tariff_rate: Some("free"),
    },

    // ═══════════════════════════════════════════════
    // Chapter 29 — v0.5 additions
    // ═══════════════════════════════════════════════
    JpRule {
        hs_code: "290124",
        jp_code: "290124000",
        jp_description: "ブタ-1,3-ジエン",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "290125",
        jp_code: "290125000",
        jp_description: "スチレン",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "290321",
        jp_code: "290321000",
        jp_description: "塩化ビニル（クロロエチレン）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "290512",
        jp_code: "290512000",
        jp_description: "プロパン-2-オール（イソプロピルアルコール）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "290539",
        jp_code: "290539000",
        jp_description: "その他のジオール（ネオペンチルグリコール等）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "290545",
        jp_code: "290545000",
        jp_description: "グリセリン",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "290549",
        jp_code: "290549000",
        jp_description: "その他の多価アルコール（TMP、ペンタエリスリトール等）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "290941",
        jp_code: "290941000",
        jp_description: "ジエチレングリコール（2,2'-オキシジエタノール）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291020",
        jp_code: "291020000",
        jp_description: "プロピレンオキサイド（1,2-エポキシプロパン）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291412",
        jp_code: "291412000",
        jp_description: "ブタノン（メチルエチルケトン）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291421",
        jp_code: "291421000",
        jp_description: "シクロヘキサノン",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291515",
        jp_code: "291515000",
        jp_description: "プロピオン酸、その塩及びエステル",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291519",
        jp_code: "291519000",
        jp_description: "その他の飽和非環式モノカルボン酸（酪酸等）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291524",
        jp_code: "291524000",
        jp_description: "酢酸ブチル",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291611",
        jp_code: "291611000",
        jp_description: "アクリル酸及びその塩",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291613",
        jp_code: "291613000",
        jp_description: "アクリル酸メチル",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291614",
        jp_code: "291614000",
        jp_description: "メタクリル酸メチル（MMA）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291619",
        jp_code: "291619000",
        jp_description: "不飽和モノカルボン酸のエステル（アクリル酸ブチル等）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291712",
        jp_code: "291712000",
        jp_description: "アジピン酸、その塩及びエステル",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291714",
        jp_code: "291714000",
        jp_description: "無水マレイン酸",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291719",
        jp_code: "291719000",
        jp_description: "その他の非環式ポリカルボン酸（コハク酸等）",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "291736",
        jp_code: "291736000",
        jp_description: "テレフタル酸及びその塩",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "292121",
        jp_code: "292121000",
        jp_description: "エチレンジアミン及びその塩",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "292122",
        jp_code: "292122000",
        jp_description: "ヘキサメチレンジアミン及びその塩",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "292610",
        jp_code: "292610000",
        jp_description: "アクリロニトリル",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "292920",
        jp_code: "292920000",
        jp_description: "イソシアナート（TDI、MDI等）",
        tariff_rate: Some("free"),
    },

    // ═══════════════════════════════════════════════
    // Chapter 38 — 各種化学工業生産品 (v0.5)
    // ═══════════════════════════════════════════════
    JpRule {
        hs_code: "380210",
        jp_code: "380210000",
        jp_description: "活性炭",
        tariff_rate: Some("free"),
    },
    JpRule {
        hs_code: "380800",
        jp_code: "380800000",
        jp_description: "殺虫剤、殺菌剤、除草剤等の調製品",
        tariff_rate: None, // complex rate depending on active ingredient
    },
    JpRule {
        hs_code: "382499",
        jp_code: "382499000",
        jp_description: "その他の化学調製品（他に分類されないもの）",
        tariff_rate: None,
    },

    // ═══════════════════════════════════════════════
    // Chapters 21, 30, 33 — special-use products (v0.5)
    // ═══════════════════════════════════════════════
    JpRule {
        hs_code: "210690",
        jp_code: "210690000",
        jp_description: "食品調製品（他に分類されないもの）",
        tariff_rate: None,
    },
    JpRule {
        hs_code: "300490",
        jp_code: "300490000",
        jp_description: "医薬品（投与量形態にしたもの等）",
        tariff_rate: None,
    },
    JpRule {
        hs_code: "330499",
        jp_code: "330499000",
        jp_description: "美容品・化粧品調製品（その他）",
        tariff_rate: None,
    },
];

/// Look up the Japan tariff code for a 6-digit HS code.
///
/// Returns `None` if no Japan-specific entry is registered for this code.
pub fn find_jp_rule(hs_code: &str) -> Option<&'static JpRule> {
    JP_RULES.iter().find(|r| r.hs_code == hs_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn naoh_solid_has_jp_code() {
        let rule = find_jp_rule("281511").unwrap();
        assert_eq!(rule.jp_code, "281511000");
        assert_eq!(rule.tariff_rate, Some("free"));
    }

    #[test]
    fn acetone_has_jp_code() {
        let rule = find_jp_rule("291411").unwrap();
        assert_eq!(rule.jp_code, "291411000");
    }

    #[test]
    fn unknown_hs_returns_none() {
        assert!(find_jp_rule("999999").is_none());
    }

    #[test]
    fn tariff_year_is_2026() {
        assert_eq!(JP_TARIFF_YEAR, 2026);
    }
}
