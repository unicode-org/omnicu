// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! Data provider struct definitions for this ICU4X component.
//!
//! Read more about data providers: [`icu_provider`]

use crate::uniset::UnicodeSet;
use std::borrow::Cow;
use std::convert::TryInto;

//
// resource key structs - the structs used directly by users of data provider
//

pub mod key {
    use icu_provider::resource_key;
    use icu_provider::ResourceKey;

    /// Macro to help define resource keys and store them in a list.
    macro_rules! define_resource_keys {
        ($count:expr; $(($k:ident, $s:literal)),+,) => {
            $( pub const $k: ResourceKey = resource_key!(uniset, $s, 1); )+
            pub const ALL_KEYS: [ResourceKey; $count] = [$($k,)+];
        };
    }

    define_resource_keys!(610;

        //
        // Binary properties
        //

        (ASCII_HEX_DIGIT_V1, "AHex"),
        (ALNUM_V1, "alnum"),
        (ALPHABETIC_V1, "Alpha"),
        (BIDI_CONTROL_V1, "Bidi_C"),
        (BIDI_MIRRORED_V1, "BidiMarker"),
        (BLANK_V1, "blank"),
        (CASED_V1, "Cased"),
        (CASE_IGNORABLE_V1, "CI"),
        (FULL_COMPOSITION_EXCLUSION_V1, "Comp_Ex"),
        (CHANGES_WHEN_CASEFOLDED_V1, "CWCF"),
        (CHANGES_WHEN_CASEMAPPED_V1, "CWCM"),
        (CHANGES_WHEN_NFKC_CASEFOLDED_V1, "CWKCF"),
        (CHANGES_WHEN_LOWERCASED_V1, "CWL"),
        (CHANGES_WHEN_TITLECASED_V1, "CWT"),
        (CHANGES_WHEN_UPPERCASED_V1, "CWU"),
        (DASH_V1, "Dash"),
        (DEPRECATED_V1, "Dep"),
        (DEFAULT_IGNORABLE_CODE_POINT_V1, "DI"),
        (DIACRITIC_V1, "Dia"),
        (EMOJI_MODIFIER_BASE_V1, "EBase"),
        (EMOJI_COMPONENT_V1, "EComp"),
        (EMOJI_MODIFIER_V1, "EMod"),
        (EMOJI_V1, "Emoji"),
        (EMOJI_PRESENTATION_V1, "EPres"),
        (EXTENDER_V1, "Ext"),
        (EXTENDED_PICTOGRAPHIC_V1, "ExtPict"),
        (GRAPH_V1, "graph"),
        (GRAPHEME_BASE_V1, "Gr_Base"),
        (GRAPHEME_EXTEND_V1, "Gr_Ext"),
        (GRAPHEME_LINK_V1, "Gr_Link"),
        (HEX_DIGIT_V1, "Hex"),
        (HYPHEN_V1, "Hyphen"),
        (ID_CONTINUE_V1, "IDC"),
        (IDEOGRAPHIC_V1, "Ideo"),
        (ID_START_V1, "IDS"),
        (IDS_BINARY_OPERATOR_V1, "IDSB"),
        (IDS_TRINARY_OPERATOR_V1, "IDST"),
        (JOIN_CONTROL_V1, "Join_C"),
        (LOGICAL_ORDER_EXCEPTION_V1, "LOE"),
        (LOWERCASE_V1, "Lower"),
        (MATH_V1, "Math"),
        (NONCHARACTER_CODE_POINT_V1, "NChar"),
        (NFC_INERT_V1, "nfcinert"),
        (NFD_INERT_V1, "nfdinert"),
        (NFKC_INERT_V1, "nfkcinert"),
        (NFKD_INERT_V1, "nfkdinert"),
        (PATTERN_SYNTAX_V1, "Pat_Syn"),
        (PATTERN_WHITE_SPACE_V1, "Pat_WS"),
        (PREPENDED_CONCATENATION_MARK_V1, "PCM"),
        (PRINT_V1, "print"),
        (QUOTATION_MARK_V1, "QMark"),
        (RADICAL_V1, "Radical"),
        (REGIONAL_INDICATOR_V1, "RI"),
        (SOFT_DOTTED_V1, "SD"),
        (SEGMENT_STARTER_V1, "segstart"),
        (CASE_SENSITIVE_V1, "Sensitive"),
        (SENTENCE_TERMINAL_V1, "STerm"),
        (TERMINAL_PUNCTUATION_V1, "Term"),
        (UNIFIED_IDEOGRAPH_V1, "UIdeo"),
        (UPPERCASE_V1, "Upper"),
        (VARIATION_SELECTOR_V1, "VS"),
        (WHITE_SPACE_V1, "WSpace"),
        (XDIGIT_V1, "xdigit"),
        (XID_CONTINUE_V1, "XIDC"),
        (XID_START_V1, "XIDS"),

        //
        // Enumerated properties
        //

        // Note: The ResourceKey subcategory strings are determined from the Rust enum
        // integer representations of the Unicode enumerated property name and the
        // Unicode enumerated property value.

        (BIDI_CLASS_ARABIC_LETTER_V1, "0=0"),
        (BIDI_CLASS_ARABIC_NUMBER_V1, "0=1"),
        (BIDI_CLASS_PARAGRAPH_SEPARATOR_V1, "0=2"),
        (BIDI_CLASS_BOUNDARY_NEUTRAL_V1, "0=3"),
        (BIDI_CLASS_COMMON_SEPARATOR_V1, "0=4"),
        (BIDI_CLASS_EUROPEAN_NUMBER_V1, "0=5"),
        (BIDI_CLASS_EUROPEAN_SEPARATOR_V1, "0=6"),
        (BIDI_CLASS_EUROPEAN_TERMINATOR_V1, "0=7"),
        (BIDI_CLASS_FIRST_STRONG_ISOLATE_V1, "0=8"),
        (BIDI_CLASS_LEFT_TO_RIGHT_V1, "0=9"),
        (BIDI_CLASS_LEFT_TO_RIGHT_EMBEDDING_V1, "0=10"),
        (BIDI_CLASS_LEFT_TO_RIGHT_ISOLATE_V1, "0=11"),
        (BIDI_CLASS_LEFT_TO_RIGHT_OVERRIDE_V1, "0=12"),
        (BIDI_CLASS_NONSPACING_MARK_V1, "0=13"),
        (BIDI_CLASS_OTHER_NEUTRAL_V1, "0=14"),
        (BIDI_CLASS_POP_DIRECTIONAL_FORMAT_V1, "0=15"),
        (BIDI_CLASS_POP_DIRECTIONAL_ISOLATE_V1, "0=16"),
        (BIDI_CLASS_RIGHT_TO_LEFT_V1, "0=17"),
        (BIDI_CLASS_RIGHT_TO_LEFT_EMBEDDING_V1, "0=18"),
        (BIDI_CLASS_RIGHT_TO_LEFT_ISOLATE_V1, "0=19"),
        (BIDI_CLASS_RIGHT_TO_LEFT_OVERRIDE_V1, "0=20"),
        (BIDI_CLASS_SEGMENT_SEPARATOR_V1, "0=21"),
        (BIDI_CLASS_WHITE_SPACE_V1, "0=22"),
        (BIDI_PAIRED_BRACKET_TYPE_CLOSE_V1, "1=0"),
        (BIDI_PAIRED_BRACKET_TYPE_NONE_V1, "1=1"),
        (BIDI_PAIRED_BRACKET_TYPE_OPEN_V1, "1=2"),
        (CANONICAL_COMBINING_CLASS_NOT_REORDERED_V1, "2=0"),
        (CANONICAL_COMBINING_CLASS_OVERLAY_V1, "2=1"),
        (CANONICAL_COMBINING_CLASS_CCC10_V1, "2=10"),
        (CANONICAL_COMBINING_CLASS_CCC103_V1, "2=103"),
        (CANONICAL_COMBINING_CLASS_CCC107_V1, "2=107"),
        (CANONICAL_COMBINING_CLASS_CCC11_V1, "2=11"),
        (CANONICAL_COMBINING_CLASS_CCC118_V1, "2=118"),
        (CANONICAL_COMBINING_CLASS_CCC12_V1, "2=12"),
        (CANONICAL_COMBINING_CLASS_CCC122_V1, "2=122"),
        (CANONICAL_COMBINING_CLASS_CCC129_V1, "2=129"),
        (CANONICAL_COMBINING_CLASS_CCC13_V1, "2=13"),
        (CANONICAL_COMBINING_CLASS_CCC130_V1, "2=130"),
        (CANONICAL_COMBINING_CLASS_CCC132_V1, "2=132"),
        (CANONICAL_COMBINING_CLASS_CCC133_V1, "2=133"),
        (CANONICAL_COMBINING_CLASS_CCC14_V1, "2=14"),
        (CANONICAL_COMBINING_CLASS_CCC15_V1, "2=15"),
        (CANONICAL_COMBINING_CLASS_CCC16_V1, "2=16"),
        (CANONICAL_COMBINING_CLASS_CCC17_V1, "2=17"),
        (CANONICAL_COMBINING_CLASS_CCC18_V1, "2=18"),
        (CANONICAL_COMBINING_CLASS_CCC19_V1, "2=19"),
        (CANONICAL_COMBINING_CLASS_CCC20_V1, "2=20"),
        (CANONICAL_COMBINING_CLASS_ATTACHED_BELOW_LEFT_V1, "2=200"),
        (CANONICAL_COMBINING_CLASS_ATTACHED_BELOW_V1, "2=202"),
        (CANONICAL_COMBINING_CLASS_CCC21_V1, "2=21"),
        (CANONICAL_COMBINING_CLASS_ATTACHED_ABOVE_V1, "2=214"),
        (CANONICAL_COMBINING_CLASS_ATTACHED_ABOVE_RIGHT_V1, "2=216"),
        (CANONICAL_COMBINING_CLASS_BELOW_LEFT_V1, "2=218"),
        (CANONICAL_COMBINING_CLASS_CCC22_V1, "2=22"),
        (CANONICAL_COMBINING_CLASS_BELOW_V1, "2=220"),
        (CANONICAL_COMBINING_CLASS_BELOW_RIGHT_V1, "2=222"),
        (CANONICAL_COMBINING_CLASS_LEFT_V1, "2=224"),
        (CANONICAL_COMBINING_CLASS_RIGHT_V1, "2=226"),
        (CANONICAL_COMBINING_CLASS_ABOVE_LEFT_V1, "2=228"),
        (CANONICAL_COMBINING_CLASS_CCC23_V1, "2=23"),
        (CANONICAL_COMBINING_CLASS_ABOVE_V1, "2=230"),
        (CANONICAL_COMBINING_CLASS_ABOVE_RIGHT_V1, "2=232"),
        (CANONICAL_COMBINING_CLASS_DOUBLE_BELOW_V1, "2=233"),
        (CANONICAL_COMBINING_CLASS_DOUBLE_ABOVE_V1, "2=234"),
        (CANONICAL_COMBINING_CLASS_CCC24_V1, "2=324"),
        (CANONICAL_COMBINING_CLASS_IOTA_SUBSCRIPT_V1, "2=240"),
        (CANONICAL_COMBINING_CLASS_CCC25_V1, "2=25"),
        (CANONICAL_COMBINING_CLASS_CCC26_V1, "2=26"),
        (CANONICAL_COMBINING_CLASS_CCC27_V1, "2=27"),
        (CANONICAL_COMBINING_CLASS_CCC28_V1, "2=28"),
        (CANONICAL_COMBINING_CLASS_CCC29_V1, "2=29"),
        (CANONICAL_COMBINING_CLASS_CCC30_V1, "2=30"),
        (CANONICAL_COMBINING_CLASS_CCC31_V1, "2=31"),
        (CANONICAL_COMBINING_CLASS_CCC32_V1, "2=32"),
        (CANONICAL_COMBINING_CLASS_CCC33_V1, "2=33"),
        (CANONICAL_COMBINING_CLASS_CCC34_V1, "2=34"),
        (CANONICAL_COMBINING_CLASS_CCC35_V1, "2=35"),
        (CANONICAL_COMBINING_CLASS_CCC36_V1, "2=36"),
        (CANONICAL_COMBINING_CLASS_HAN_READING_V1, "2=6"),
        (CANONICAL_COMBINING_CLASS_NUKTA_V1, "2=7"),
        (CANONICAL_COMBINING_CLASS_KANA_VOICING_V1, "2=8"),
        (CANONICAL_COMBINING_CLASS_CCC84_V1, "2=84"),
        (CANONICAL_COMBINING_CLASS_VIRAMA_V1, "2=9"),
        (CANONICAL_COMBINING_CLASS_CCC91_V1, "2=91"),
        (DECOMPOSITION_TYPE_CAN_V1, "3=0"),
        (DECOMPOSITION_TYPE_COM_V1, "3=1"),
        (DECOMPOSITION_TYPE_ENC_V1, "3=2"),
        (DECOMPOSITION_TYPE_FIN_V1, "3=3"),
        (DECOMPOSITION_TYPE_FONT_V1, "3=4"),
        (DECOMPOSITION_TYPE_FRA_V1, "3=5"),
        (DECOMPOSITION_TYPE_INIT_V1, "3=6"),
        (DECOMPOSITION_TYPE_ISO_V1, "3=7"),
        (DECOMPOSITION_TYPE_MED_V1, "3=8"),
        (DECOMPOSITION_TYPE_NAR_V1, "3=9"),
        (DECOMPOSITION_TYPE_NB_V1, "3=10"),
        (DECOMPOSITION_TYPE_NONE_V1, "3=11"),
        (DECOMPOSITION_TYPE_SML_V1, "3=12"),
        (DECOMPOSITION_TYPE_SQR_V1, "3=13"),
        (DECOMPOSITION_TYPE_SUB_V1, "3=14"),
        (DECOMPOSITION_TYPE_SUP_V1, "3=15"),
        (DECOMPOSITION_TYPE_VERT_V1, "3=16"),
        (DECOMPOSITION_TYPE_WIDE_V1, "3=17"),
        (EAST_ASIAN_WIDTH_AMBIGUOUS_V1, "4=0"),
        (EAST_ASIAN_WIDTH_FULLWIDTH_V1, "4=1"),
        (EAST_ASIAN_WIDTH_HALFWIDTH_V1, "4=2"),
        (EAST_ASIAN_WIDTH_NEUTRAL_V1, "4=3"),
        (EAST_ASIAN_WIDTH_NARROW_V1, "4=4"),
        (EAST_ASIAN_WIDTH_WIDE_V1, "4=5"),
        (GENERAL_CATEGORY_OTHER_V1, "5=0"),
        (GENERAL_CATEGORY_CNTRL_V1, "5=1"),
        (GENERAL_CATEGORY_FORMAT_V1, "5=2"),
        (GENERAL_CATEGORY_UNASSIGNED_V1, "5=3"),
        (GENERAL_CATEGORY_PRIVATE_USE_V1, "5=4"),
        (GENERAL_CATEGORY_SURROGATE_V1, "5=5"),
        (GENERAL_CATEGORY_LETTER_V1, "5=6"),
        (GENERAL_CATEGORY_CASED_LETTER_V1, "5=7"),
        (GENERAL_CATEGORY_LOWERCASE_LETTER_V1, "5=8"),
        (GENERAL_CATEGORY_MODIFIER_LETTER_V1, "5=9"),
        (GENERAL_CATEGORY_OTHER_LETTER_V1, "5=10"),
        (GENERAL_CATEGORY_TITLECASE_LETTER_V1, "5=11"),
        (GENERAL_CATEGORY_UPPERCASE_LETTER_V1, "5=12"),
        (GENERAL_CATEGORY_COMBINING_MARK_V1, "5=13"),
        (GENERAL_CATEGORY_SPACING_MARK_V1, "5=14"),
        (GENERAL_CATEGORY_ENCLOSING_MARK_V1, "5=15"),
        (GENERAL_CATEGORY_NONSPACING_MARK_V1, "5=16"),
        (GENERAL_CATEGORY_NUMBER_V1, "5=17"),
        (GENERAL_CATEGORY_DIGIT_V1, "5=18"),
        (GENERAL_CATEGORY_LETTER_NUMBER_V1, "5=19"),
        (GENERAL_CATEGORY_OTHER_NUMBER_V1, "5=20"),
        (GENERAL_CATEGORY_PUNCT_V1, "5=21"),
        (GENERAL_CATEGORY_CONNECTOR_PUNCTUATION_V1, "5=22"),
        (GENERAL_CATEGORY_DASH_PUNCTUATION_V1, "5=23"),
        (GENERAL_CATEGORY_CLOSE_PUNCTUATION_V1, "5=24"),
        (GENERAL_CATEGORY_FINAL_PUNCTUATION_V1, "5=25"),
        (GENERAL_CATEGORY_INITIAL_PUNCTUATION_V1, "5=26"),
        (GENERAL_CATEGORY_OTHER_PUNCTUATION_V1, "5=27"),
        (GENERAL_CATEGORY_OPEN_PUNCTUATION_V1, "5=28"),
        (GENERAL_CATEGORY_SYMBOL_V1, "5=29"),
        (GENERAL_CATEGORY_CURRENCY_SYMBOL_V1, "5=30"),
        (GENERAL_CATEGORY_MODIFIER_SYMBOL_V1, "5=31"),
        (GENERAL_CATEGORY_MATH_SYMBOL_V1, "5=32"),
        (GENERAL_CATEGORY_OTHER_SYMBOL_V1, "5=33"),
        (GENERAL_CATEGORY_SEPARATOR_V1, "5=34"),
        (GENERAL_CATEGORY_LINE_SEPARATOR_V1, "5=35"),
        (GENERAL_CATEGORY_PARAGRAPH_SEPARATOR_V1, "5=36"),
        (GENERAL_CATEGORY_SPACE_SEPARATOR_V1, "5=37"),
        (GRAPHEME_CLUSTER_BREAK_CONTROL_V1, "6=0"),
        (GRAPHEME_CLUSTER_BREAK_CR_V1, "6=1"),
        (GRAPHEME_CLUSTER_BREAK_E_BASE_V1, "6=2"),
        (GRAPHEME_CLUSTER_BREAK_E_BASE_GAZ_V1, "6=3"),
        (GRAPHEME_CLUSTER_BREAK_E_MODIFIER_V1, "6=4"),
        (GRAPHEME_CLUSTER_BREAK_EXTEND_V1, "6=5"),
        (GRAPHEME_CLUSTER_BREAK_GLUE_AFTER_ZWJ_V1, "6=6"),
        (GRAPHEME_CLUSTER_BREAK_L_V1, "6=7"),
        (GRAPHEME_CLUSTER_BREAK_LF_V1, "6=8"),
        (GRAPHEME_CLUSTER_BREAK_LV_V1, "6=9"),
        (GRAPHEME_CLUSTER_BREAK_LVT_V1, "6=10"),
        (GRAPHEME_CLUSTER_BREAK_PREPEND_V1, "6=11"),
        (GRAPHEME_CLUSTER_BREAK_REGIONAL_INDICATOR_V1, "6=12"),
        (GRAPHEME_CLUSTER_BREAK_SPACINGMARK_V1, "6=13"),
        (GRAPHEME_CLUSTER_BREAK_T_V1, "6=14"),
        (GRAPHEME_CLUSTER_BREAK_V_V1, "6=15"),
        (GRAPHEME_CLUSTER_BREAK_OTHER_V1, "6=16"),
        (GRAPHEME_CLUSTER_BREAK_ZWJ_V1, "6=17"),
        (HANGUL_SYLLABLE_TYPE_LEADING_JAMO_V1, "7=0"),
        (HANGUL_SYLLABLE_TYPE_LV_SYLLABLE_V1, "7=1"),
        (HANGUL_SYLLABLE_TYPE_LVT_SYLLABLE_V1, "7=2"),
        (HANGUL_SYLLABLE_TYPE_NOT_APPLICABLE_V1, "7=3"),
        (HANGUL_SYLLABLE_TYPE_TRAILING_JAMO_V1, "7=4"),
        (HANGUL_SYLLABLE_TYPE_VOWEL_JAMO_V1, "7=5"),
        (INDIC_POSITIONAL_CATEGORY_BOTTOM_V1, "8=0"),
        (INDIC_POSITIONAL_CATEGORY_BOTTOM_AND_LEFT_V1, "8=1"),
        (INDIC_POSITIONAL_CATEGORY_BOTTOM_AND_RIGHT_V1, "8=2"),
        (INDIC_POSITIONAL_CATEGORY_LEFT_V1, "8=3"),
        (INDIC_POSITIONAL_CATEGORY_LEFT_AND_RIGHT_V1, "8=4"),
        (INDIC_POSITIONAL_CATEGORY_NA_V1, "8=5"),
        (INDIC_POSITIONAL_CATEGORY_OVERSTRUCK_V1, "8=6"),
        (INDIC_POSITIONAL_CATEGORY_RIGHT_V1, "8=7"),
        (INDIC_POSITIONAL_CATEGORY_TOP_V1, "8=8"),
        (INDIC_POSITIONAL_CATEGORY_TOP_AND_BOTTOM_V1, "8=9"),
        (INDIC_POSITIONAL_CATEGORY_TOP_AND_BOTTOM_AND_LEFT_V1, "8=10"),
        (INDIC_POSITIONAL_CATEGORY_TOP_AND_BOTTOM_AND_RIGHT_V1, "8=11"),
        (INDIC_POSITIONAL_CATEGORY_TOP_AND_LEFT_V1, "8=12"),
        (INDIC_POSITIONAL_CATEGORY_TOP_AND_LEFT_AND_RIGHT_V1, "8=13"),
        (INDIC_POSITIONAL_CATEGORY_TOP_AND_RIGHT_V1, "8=14"),
        (INDIC_POSITIONAL_CATEGORY_VISUAL_ORDER_LEFT_V1, "8=15"),
        (INDIC_SYLLABIC_CATEGORY_AVAGRAHA_V1, "9=0"),
        (INDIC_SYLLABIC_CATEGORY_BINDU_V1, "9=1"),
        (INDIC_SYLLABIC_CATEGORY_BRAHMI_JOINING_NUMBER_V1, "9=2"),
        (INDIC_SYLLABIC_CATEGORY_CANTILLATION_MARK_V1, "9=3"),
        (INDIC_SYLLABIC_CATEGORY_CONSONANT_V1, "9=4"),
        (INDIC_SYLLABIC_CATEGORY_CONSONANT_DEAD_V1, "9=5"),
        (INDIC_SYLLABIC_CATEGORY_CONSONANT_FINAL_V1, "9=6"),
        (INDIC_SYLLABIC_CATEGORY_CONSONANT_HEAD_LETTER_V1, "9=7"),
        (INDIC_SYLLABIC_CATEGORY_CONSONANT_INITIAL_POSTFIXED_V1, "9=8"),
        (INDIC_SYLLABIC_CATEGORY_CONSONANT_KILLER_V1, "9=9"),
        (INDIC_SYLLABIC_CATEGORY_CONSONANT_MEDIAL_V1, "9=10"),
        (INDIC_SYLLABIC_CATEGORY_CONSONANT_PLACEHOLDER_V1, "9=11"),
        (INDIC_SYLLABIC_CATEGORY_CONSONANT_PRECEDING_REPHA_V1, "9=12"),
        (INDIC_SYLLABIC_CATEGORY_CONSONANT_PREFIXED_V1, "9=13"),
        (INDIC_SYLLABIC_CATEGORY_CONSONANT_SUBJOINED_V1, "9=14"),
        (INDIC_SYLLABIC_CATEGORY_CONSONANT_SUCCEEDING_REPHA_V1, "9=15"),
        (INDIC_SYLLABIC_CATEGORY_CONSONANT_WITH_STACKER_V1, "9=16"),
        (INDIC_SYLLABIC_CATEGORY_GEMINATION_MARK_V1, "9=17"),
        (INDIC_SYLLABIC_CATEGORY_INVISIBLE_STACKER_V1, "9=18"),
        (INDIC_SYLLABIC_CATEGORY_JOINER_V1, "9=19"),
        (INDIC_SYLLABIC_CATEGORY_MODIFYING_LETTER_V1, "9=20"),
        (INDIC_SYLLABIC_CATEGORY_NON_JOINER_V1, "9=21"),
        (INDIC_SYLLABIC_CATEGORY_NUKTA_V1, "9=22"),
        (INDIC_SYLLABIC_CATEGORY_NUMBER_V1, "9=23"),
        (INDIC_SYLLABIC_CATEGORY_NUMBER_JOINER_V1, "9=24"),
        (INDIC_SYLLABIC_CATEGORY_OTHER_V1, "9=25"),
        (INDIC_SYLLABIC_CATEGORY_PURE_KILLER_V1, "9=26"),
        (INDIC_SYLLABIC_CATEGORY_REGISTER_SHIFTER_V1, "9=27"),
        (INDIC_SYLLABIC_CATEGORY_SYLLABLE_MODIFIER_V1, "9=28"),
        (INDIC_SYLLABIC_CATEGORY_TONE_LETTER_V1, "9=29"),
        (INDIC_SYLLABIC_CATEGORY_TONE_MARK_V1, "9=30"),
        (INDIC_SYLLABIC_CATEGORY_VIRAMA_V1, "9=31"),
        (INDIC_SYLLABIC_CATEGORY_VISARGA_V1, "9=32"),
        (INDIC_SYLLABIC_CATEGORY_VOWEL_V1, "9=33"),
        (INDIC_SYLLABIC_CATEGORY_VOWEL_DEPENDENT_V1, "9=34"),
        (INDIC_SYLLABIC_CATEGORY_VOWEL_INDEPENDENT_V1, "9=35"),
        (JOINING_GROUP_AFRICAN_FEH_V1, "10=0"),
        (JOINING_GROUP_AFRICAN_NOON_V1, "10=1"),
        (JOINING_GROUP_AFRICAN_QAF_V1, "10=2"),
        (JOINING_GROUP_AIN_V1, "10=3"),
        (JOINING_GROUP_ALAPH_V1, "10=4"),
        (JOINING_GROUP_ALEF_V1, "10=5"),
        (JOINING_GROUP_BEH_V1, "10=6"),
        (JOINING_GROUP_BETH_V1, "10=7"),
        (JOINING_GROUP_BURUSHASKI_YEH_BARREE_V1, "10=8"),
        (JOINING_GROUP_DAL_V1, "10=9"),
        (JOINING_GROUP_DALATH_RISH_V1, "10=10"),
        (JOINING_GROUP_E_V1, "10=11"),
        (JOINING_GROUP_FARSI_YEH_V1, "10=12"),
        (JOINING_GROUP_FE_V1, "10=13"),
        (JOINING_GROUP_FEH_V1, "10=14"),
        (JOINING_GROUP_FINAL_SEMKATH_V1, "10=15"),
        (JOINING_GROUP_GAF_V1, "10=16"),
        (JOINING_GROUP_GAMAL_V1, "10=17"),
        (JOINING_GROUP_HAH_V1, "10=18"),
        (JOINING_GROUP_HANIFI_ROHINGYA_KINNA_YA_V1, "10=19"),
        (JOINING_GROUP_HANIFI_ROHINGYA_PA_V1, "10=20"),
        (JOINING_GROUP_HE_V1, "10=21"),
        (JOINING_GROUP_HEH_V1, "10=22"),
        (JOINING_GROUP_HEH_GOAL_V1, "10=23"),
        (JOINING_GROUP_HETH_V1, "10=24"),
        (JOINING_GROUP_KAF_V1, "10=25"),
        (JOINING_GROUP_KAPH_V1, "10=26"),
        (JOINING_GROUP_KHAPH_V1, "10=27"),
        (JOINING_GROUP_KNOTTED_HEH_V1, "10=28"),
        (JOINING_GROUP_LAM_V1, "10=29"),
        (JOINING_GROUP_LAMADH_V1, "10=30"),
        (JOINING_GROUP_MALAYALAM_BHA_V1, "10=31"),
        (JOINING_GROUP_MALAYALAM_JA_V1, "10=32"),
        (JOINING_GROUP_MALAYALAM_LLA_V1, "10=33"),
        (JOINING_GROUP_MALAYALAM_LLLA_V1, "10=34"),
        (JOINING_GROUP_MALAYALAM_NGA_V1, "10=35"),
        (JOINING_GROUP_MALAYALAM_NNA_V1, "10=36"),
        (JOINING_GROUP_MALAYALAM_NNNA_V1, "10=37"),
        (JOINING_GROUP_MALAYALAM_NYA_V1, "10=38"),
        (JOINING_GROUP_MALAYALAM_RA_V1, "10=39"),
        (JOINING_GROUP_MALAYALAM_SSA_V1, "10=40"),
        (JOINING_GROUP_MALAYALAM_TTA_V1, "10=41"),
        (JOINING_GROUP_MANICHAEAN_ALEPH_V1, "10=42"),
        (JOINING_GROUP_MANICHAEAN_AYIN_V1, "10=43"),
        (JOINING_GROUP_MANICHAEAN_BETH_V1, "10=44"),
        (JOINING_GROUP_MANICHAEAN_DALETH_V1, "10=45"),
        (JOINING_GROUP_MANICHAEAN_DHAMEDH_V1, "10=46"),
        (JOINING_GROUP_MANICHAEAN_FIVE_V1, "10=47"),
        (JOINING_GROUP_MANICHAEAN_GIMEL_V1, "10=48"),
        (JOINING_GROUP_MANICHAEAN_HETH_V1, "10=49"),
        (JOINING_GROUP_MANICHAEAN_HUNDRED_V1, "10=50"),
        (JOINING_GROUP_MANICHAEAN_KAPH_V1, "10=51"),
        (JOINING_GROUP_MANICHAEAN_LAMEDH_V1, "10=52"),
        (JOINING_GROUP_MANICHAEAN_MEM_V1, "10=53"),
        (JOINING_GROUP_MANICHAEAN_NUN_V1, "10=54"),
        (JOINING_GROUP_MANICHAEAN_ONE_V1, "10=55"),
        (JOINING_GROUP_MANICHAEAN_PE_V1, "10=56"),
        (JOINING_GROUP_MANICHAEAN_QOPH_V1, "10=57"),
        (JOINING_GROUP_MANICHAEAN_RESH_V1, "10=58"),
        (JOINING_GROUP_MANICHAEAN_SADHE_V1, "10=59"),
        (JOINING_GROUP_MANICHAEAN_SAMEKH_V1, "10=60"),
        (JOINING_GROUP_MANICHAEAN_TAW_V1, "10=61"),
        (JOINING_GROUP_MANICHAEAN_TEN_V1, "10=62"),
        (JOINING_GROUP_MANICHAEAN_TETH_V1, "10=63"),
        (JOINING_GROUP_MANICHAEAN_THAMEDH_V1, "10=64"),
        (JOINING_GROUP_MANICHAEAN_TWENTY_V1, "10=65"),
        (JOINING_GROUP_MANICHAEAN_WAW_V1, "10=66"),
        (JOINING_GROUP_MANICHAEAN_YODH_V1, "10=67"),
        (JOINING_GROUP_MANICHAEAN_ZAYIN_V1, "10=68"),
        (JOINING_GROUP_MEEM_V1, "10=69"),
        (JOINING_GROUP_MIM_V1, "10=70"),
        (JOINING_GROUP_NO_JOINING_GROUP_V1, "10=71"),
        (JOINING_GROUP_NOON_V1, "10=72"),
        (JOINING_GROUP_NUN_V1, "10=73"),
        (JOINING_GROUP_NYA_V1, "10=74"),
        (JOINING_GROUP_PE_V1, "10=75"),
        (JOINING_GROUP_QAF_V1, "10=76"),
        (JOINING_GROUP_QAPH_V1, "10=77"),
        (JOINING_GROUP_REH_V1, "10=78"),
        (JOINING_GROUP_REVERSED_PE_V1, "10=79"),
        (JOINING_GROUP_ROHINGYA_YEH_V1, "10=80"),
        (JOINING_GROUP_SAD_V1, "10=81"),
        (JOINING_GROUP_SADHE_V1, "10=82"),
        (JOINING_GROUP_SEEN_V1, "10=83"),
        (JOINING_GROUP_SEMKATH_V1, "10=84"),
        (JOINING_GROUP_SHIN_V1, "10=85"),
        (JOINING_GROUP_STRAIGHT_WAW_V1, "10=86"),
        (JOINING_GROUP_SWASH_KAF_V1, "10=87"),
        (JOINING_GROUP_SYRIAC_WAW_V1, "10=88"),
        (JOINING_GROUP_TAH_V1, "10=89"),
        (JOINING_GROUP_TAW_V1, "10=90"),
        (JOINING_GROUP_TEH_MARBUTA_V1, "10=91"),
        (JOINING_GROUP_HAMZA_ON_HEH_GOAL_V1, "10=92"),
        (JOINING_GROUP_TETH_V1, "10=93"),
        (JOINING_GROUP_WAW_V1, "10=94"),
        (JOINING_GROUP_YEH_V1, "10=95"),
        (JOINING_GROUP_YEH_BARREE_V1, "10=96"),
        (JOINING_GROUP_YEH_WITH_TAIL_V1, "10=97"),
        (JOINING_GROUP_YUDH_V1, "10=98"),
        (JOINING_GROUP_YUDH_HE_V1, "10=99"),
        (JOINING_GROUP_ZAIN_V1, "10=100"),
        (JOINING_GROUP_ZHAIN_V1, "10=101"),
        (JOINING_TYPE_JOIN_CAUSING_V1, "11=0"),
        (JOINING_TYPE_DUAL_JOINING_V1, "11=1"),
        (JOINING_TYPE_LEFT_JOINING_V1, "11=2"),
        (JOINING_TYPE_RIGHT_JOINING_V1, "11=3"),
        (JOINING_TYPE_TRANSPARENT_V1, "11=4"),
        (JOINING_TYPE_NON_JOINING_V1, "11=5"),
        (LINE_BREAK_AMBIGUOUS_V1, "12=0"),
        (LINE_BREAK_ALPHABETIC_V1, "12=1"),
        (LINE_BREAK_BREAK_BOTH_V1, "12=2"),
        (LINE_BREAK_BREAK_AFTER_V1, "12=3"),
        (LINE_BREAK_BREAK_BEFORE_V1, "12=4"),
        (LINE_BREAK_MANDATORY_BREAK_V1, "12=5"),
        (LINE_BREAK_CONTINGENT_BREAK_V1, "12=6"),
        (LINE_BREAK_CONDITIONAL_JAPANESE_STARTER_V1, "12=7"),
        (LINE_BREAK_CLOSE_PUNCTUATION_V1, "12=8"),
        (LINE_BREAK_COMBINING_MARK_V1, "12=9"),
        (LINE_BREAK_CLOSE_PARENTHESIS_V1, "12=10"),
        (LINE_BREAK_CARRIAGE_RETURN_V1, "12=11"),
        (LINE_BREAK_E_BASE_V1, "12=12"),
        (LINE_BREAK_E_MODIFIER_V1, "12=13"),
        (LINE_BREAK_EXCLAMATION_V1, "12=14"),
        (LINE_BREAK_GLUE_V1, "12=15"),
        (LINE_BREAK_H2_V1, "12=16"),
        (LINE_BREAK_H3_V1, "12=17"),
        (LINE_BREAK_HEBREW_LETTER_V1, "12=18"),
        (LINE_BREAK_HYPHEN_V1, "12=19"),
        (LINE_BREAK_IDEOGRAPHIC_V1, "12=20"),
        (LINE_BREAK_INSEPERABLE_V1, "12=21"),
        (LINE_BREAK_INFIX_NUMERIC_V1, "12=22"),
        (LINE_BREAK_JL_V1, "12=23"),
        (LINE_BREAK_JT_V1, "12=24"),
        (LINE_BREAK_JV_V1, "12=25"),
        (LINE_BREAK_LINE_FEED_V1, "12=26"),
        (LINE_BREAK_NEXT_LINE_V1, "12=27"),
        (LINE_BREAK_NONSTARTER_V1, "12=28"),
        (LINE_BREAK_NUMERIC_V1, "12=29"),
        (LINE_BREAK_OPEN_PUNCTUATION_V1, "12=30"),
        (LINE_BREAK_POSTFIX_NUMERIC_V1, "12=31"),
        (LINE_BREAK_PREFIX_NUMERIC_V1, "12=32"),
        (LINE_BREAK_QUOTATION_V1, "12=33"),
        (LINE_BREAK_REGIONAL_INDICATOR_V1, "12=34"),
        (LINE_BREAK_COMPLEX_CONTEXT_V1, "12=35"),
        (LINE_BREAK_SURROGATE_V1, "12=36"),
        (LINE_BREAK_SPACE_V1, "12=37"),
        (LINE_BREAK_BREAK_SYMBOLS_V1, "12=38"),
        (LINE_BREAK_WORD_JOINER_V1, "12=39"),
        (LINE_BREAK_UNKNOWN_V1, "12=40"),
        (LINE_BREAK_ZWSPACE_V1, "12=41"),
        (LINE_BREAK_ZWJ_V1, "12=42"),
        (LEAD_CANONICAL_COMBINING_CLASS_NOT_REORDERED_V1, "13=0"),
        (LEAD_CANONICAL_COMBINING_CLASS_OVERLAY_V1, "13=1"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC10_V1, "13=10"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC103_V1, "13=103"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC107_V1, "13=107"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC11_V1, "13=11"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC118_V1, "13=118"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC12_V1, "13=12"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC122_V1, "13=122"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC129_V1, "13=129"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC13_V1, "13=13"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC130_V1, "13=130"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC132_V1, "13=132"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC133_V1, "13=133"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC14_V1, "13=14"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC15_V1, "13=15"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC16_V1, "13=16"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC17_V1, "13=17"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC18_V1, "13=18"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC19_V1, "13=19"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC20_V1, "13=20"),
        (LEAD_CANONICAL_COMBINING_CLASS_ATTACHED_BELOW_LEFT_V1, "13=200"),
        (LEAD_CANONICAL_COMBINING_CLASS_ATTACHED_BELOW_V1, "13=202"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC21_V1, "13=21"),
        (LEAD_CANONICAL_COMBINING_CLASS_ATTACHED_ABOVE_V1, "13=214"),
        (LEAD_CANONICAL_COMBINING_CLASS_ATTACHED_ABOVE_RIGHT_V1, "13=216"),
        (LEAD_CANONICAL_COMBINING_CLASS_BELOW_LEFT_V1, "13=218"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC22_V1, "13=22"),
        (LEAD_CANONICAL_COMBINING_CLASS_BELOW_V1, "13=220"),
        (LEAD_CANONICAL_COMBINING_CLASS_BELOW_RIGHT_V1, "13=222"),
        (LEAD_CANONICAL_COMBINING_CLASS_LEFT_V1, "13=224"),
        (LEAD_CANONICAL_COMBINING_CLASS_RIGHT_V1, "13=226"),
        (LEAD_CANONICAL_COMBINING_CLASS_ABOVE_LEFT_V1, "13=228"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC23_V1, "13=23"),
        (LEAD_CANONICAL_COMBINING_CLASS_ABOVE_V1, "13=230"),
        (LEAD_CANONICAL_COMBINING_CLASS_ABOVE_RIGHT_V1, "13=232"),
        (LEAD_CANONICAL_COMBINING_CLASS_DOUBLE_BELOW_V1, "13=233"),
        (LEAD_CANONICAL_COMBINING_CLASS_DOUBLE_ABOVE_V1, "13=234"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC24_V1, "13=24"),
        (LEAD_CANONICAL_COMBINING_CLASS_IOTA_SUBSCRIPT_V1, "13=240"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC25_V1, "13=25"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC26_V1, "13=26"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC27_V1, "13=27"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC28_V1, "13=28"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC29_V1, "13=29"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC30_V1, "13=30"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC31_V1, "13=31"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC32_V1, "13=32"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC33_V1, "13=33"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC34_V1, "13=34"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC35_V1, "13=35"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC36_V1, "13=36"),
        (LEAD_CANONICAL_COMBINING_CLASS_HAN_READING_V1, "13=6"),
        (LEAD_CANONICAL_COMBINING_CLASS_NUKTA_V1, "13=7"),
        (LEAD_CANONICAL_COMBINING_CLASS_KANA_VOICING_V1, "13=8"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC84_V1, "13=84"),
        (LEAD_CANONICAL_COMBINING_CLASS_VIRAMA_V1, "13=9"),
        (LEAD_CANONICAL_COMBINING_CLASS_CCC91_V1, "13=91"),
        (NFC_QUICK_CHECK_MAYBE_V1, "14=0"),
        (NFC_QUICK_CHECK_NO_V1, "14=1"),
        (NFC_QUICK_CHECK_YES_V1, "14=2"),
        (NFD_QUICK_CHECK_NO_V1, "15=0"),
        (NFD_QUICK_CHECK_YES_V1, "15=1"),
        (NFKC_QUICK_CHECK_MAYBE_V1, "16=0"),
        (NFKC_QUICK_CHECK_NO_V1, "16=1"),
        (NFKC_QUICK_CHECK_YES_V1, "16=2"),
        (NFKD_QUICK_CHECK_NO_V1, "17=0"),
        (NFKD_QUICK_CHECK_YES_V1, "17=1"),
        (NUMERIC_TYPE_DECIMAL_V1, "18=0"),
        (NUMERIC_TYPE_DIGIT_V1, "18=1"),
        (NUMERIC_TYPE_NONE_V1, "18=2"),
        (NUMERIC_TYPE_NUMERIC_V1, "18=3"),
        (SENTENCE_BREAK_ATERM_V1, "19=0"),
        (SENTENCE_BREAK_CLOSE_V1, "19=1"),
        (SENTENCE_BREAK_CR_V1, "19=2"),
        (SENTENCE_BREAK_EXTEND_V1, "19=3"),
        (SENTENCE_BREAK_FORMAT_V1, "19=4"),
        (SENTENCE_BREAK_OLETTER_V1, "19=5"),
        (SENTENCE_BREAK_LF_V1, "19=6"),
        (SENTENCE_BREAK_LOWER_V1, "19=7"),
        (SENTENCE_BREAK_NUMERIC_V1, "19=8"),
        (SENTENCE_BREAK_SCONTINUE_V1, "19=9"),
        (SENTENCE_BREAK_SEP_V1, "19=10"),
        (SENTENCE_BREAK_SP_V1, "19=11"),
        (SENTENCE_BREAK_STERM_V1, "19=12"),
        (SENTENCE_BREAK_UPPER_V1, "19=13"),
        (SENTENCE_BREAK_OTHER_V1, "19=14"),
        (TRAIL_CANONICAL_COMBINING_CLASS_NOT_REORDERED_V1, "20=0"),
        (TRAIL_CANONICAL_COMBINING_CLASS_OVERLAY_V1, "20=1"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC10_V1, "20=10"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC103_V1, "20=103"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC107_V1, "20=107"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC11_V1, "20=11"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC118_V1, "20=118"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC12_V1, "20=12"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC122_V1, "20=122"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC129_V1, "20=129"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC13_V1, "20=13"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC130_V1, "20=130"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC132_V1, "20=132"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC133_V1, "20=133"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC14_V1, "20=14"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC15_V1, "20=15"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC16_V1, "20=16"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC17_V1, "20=17"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC18_V1, "20=18"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC19_V1, "20=19"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC20_V1, "20=20"),
        (TRAIL_CANONICAL_COMBINING_CLASS_ATTACHED_BELOW_LEFT_V1, "20=200"),
        (TRAIL_CANONICAL_COMBINING_CLASS_ATTACHED_BELOW_V1, "20=202"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC21_V1, "20=21"),
        (TRAIL_CANONICAL_COMBINING_CLASS_ATTACHED_ABOVE_V1, "20=214"),
        (TRAIL_CANONICAL_COMBINING_CLASS_ATTACHED_ABOVE_RIGHT_V1, "20=216"),
        (TRAIL_CANONICAL_COMBINING_CLASS_BELOW_LEFT_V1, "20=218"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC22_V1, "20=22"),
        (TRAIL_CANONICAL_COMBINING_CLASS_BELOW_V1, "20=220"),
        (TRAIL_CANONICAL_COMBINING_CLASS_BELOW_RIGHT_V1, "20=222"),
        (TRAIL_CANONICAL_COMBINING_CLASS_LEFT_V1, "20=224"),
        (TRAIL_CANONICAL_COMBINING_CLASS_RIGHT_V1, "20=226"),
        (TRAIL_CANONICAL_COMBINING_CLASS_ABOVE_LEFT_V1, "20=228"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC23_V1, "20=23"),
        (TRAIL_CANONICAL_COMBINING_CLASS_ABOVE_V1, "20=230"),
        (TRAIL_CANONICAL_COMBINING_CLASS_ABOVE_RIGHT_V1, "20=232"),
        (TRAIL_CANONICAL_COMBINING_CLASS_DOUBLE_BELOW_V1, "20=233"),
        (TRAIL_CANONICAL_COMBINING_CLASS_DOUBLE_ABOVE_V1, "20=234"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC24_V1, "20=24"),
        (TRAIL_CANONICAL_COMBINING_CLASS_IOTA_SUBSCRIPT_V1, "20=240"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC25_V1, "20=25"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC26_V1, "20=26"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC27_V1, "20=27"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC28_V1, "20=28"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC29_V1, "20=29"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC30_V1, "20=30"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC31_V1, "20=31"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC32_V1, "20=32"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC33_V1, "20=33"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC34_V1, "20=34"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC35_V1, "20=35"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC36_V1, "20=36"),
        (TRAIL_CANONICAL_COMBINING_CLASS_HAN_READING_V1, "20=6"),
        (TRAIL_CANONICAL_COMBINING_CLASS_NUKTA_V1, "20=7"),
        (TRAIL_CANONICAL_COMBINING_CLASS_KANA_VOICING_V1, "20=8"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC84_V1, "20=84"),
        (TRAIL_CANONICAL_COMBINING_CLASS_VIRAMA_V1, "20=9"),
        (TRAIL_CANONICAL_COMBINING_CLASS_CCC91_V1, "20=91"),
        (VERTICAL_ORIENTATION_ROTATED_V1, "21=0"),
        (VERTICAL_ORIENTATION_TRANSFORMED_ROTATED_V1, "21=1"),
        (VERTICAL_ORIENTATION_TRANSFORMED_UPRIGHT_V1, "21=2"),
        (VERTICAL_ORIENTATION_UPRIGHT_V1, "21=3"),
        (WORD_BREAK_CR_V1, "22=0"),
        (WORD_BREAK_DOUBLE_QUOTE_V1, "22=1"),
        (WORD_BREAK_E_BASE_V1, "22=2"),
        (WORD_BREAK_E_BASE_GAZ_V1, "22=3"),
        (WORD_BREAK_E_MODIFIER_V1, "22=4"),
        (WORD_BREAK_EXTENDNUMLET_V1, "22=5"),
        (WORD_BREAK_EXTEND_V1, "22=6"),
        (WORD_BREAK_FORMAT_V1, "22=7"),
        (WORD_BREAK_GLUE_AFTER_ZWJ_V1, "22=8"),
        (WORD_BREAK_HEBREW_LETTER_V1, "22=9"),
        (WORD_BREAK_KATAKANA_V1, "22=10"),
        (WORD_BREAK_ALETTER_V1, "22=11"),
        (WORD_BREAK_LF_V1, "22=12"),
        (WORD_BREAK_MIDNUMLET_V1, "22=13"),
        (WORD_BREAK_MIDLETTER_V1, "22=14"),
        (WORD_BREAK_MIDNUM_V1, "22=15"),
        (WORD_BREAK_NEWLINE_V1, "22=16"),
        (WORD_BREAK_NUMERIC_V1, "22=17"),
        (WORD_BREAK_REGIONAL_INDICATOR_V1, "22=18"),
        (WORD_BREAK_SINGLE_QUOTE_V1, "22=19"),
        (WORD_BREAK_WSEGSPACE_V1, "22=20"),
        (WORD_BREAK_OTHER_V1, "22=21"),
        (WORD_BREAK_ZWJ_V1, "22=22"),
    );
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "provider_serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnicodeProperty<'s> {
    pub name: Cow<'s, str>,
    pub inv_list: Vec<u32>,
}

icu_provider::unsafe_impl_data_marker_with_lifetime!(
    UnicodeProperty<'s>,
    /// Marker type for [`UnicodeProperty`]
    UnicodePropertyMarker,
    TEMP_ZCF
);

impl Default for UnicodeProperty<'static> {
    /// Default empty nameless property
    fn default() -> UnicodeProperty<'static> {
        UnicodeProperty {
            name: Cow::Borrowed(""),
            inv_list: vec![],
        }
    }
}

impl<'s> UnicodeProperty<'s> {
    pub fn from_uniset(set: &UnicodeSet, name: Cow<'s, str>) -> UnicodeProperty<'s> {
        let inv_list = set.get_inversion_list();
        UnicodeProperty { name, inv_list }
    }
}

impl<'s> TryInto<UnicodeSet> for UnicodeProperty<'s> {
    type Error = crate::UnicodeSetError;
    fn try_into(self) -> Result<UnicodeSet, Self::Error> {
        UnicodeSet::from_inversion_list(self.inv_list)
    }
}
