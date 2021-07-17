// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! Data provider struct definitions for this ICU4X component.
//!
//! Read more about data providers: [`icu_provider`]

use crate::builder::UnicodeSetBuilder;
use crate::uniset::UnicodeSet;
use icu_provider::yoke::{self, *};
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
            $( pub const $k: ResourceKey = resource_key!(UnicodeSet, $s, 1); )+
            pub const ALL_KEYS: [ResourceKey; $count] = [$($k,)+];
        };
    }

    define_resource_keys!(95;

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

        // Note: The ResourceKey subcategory strings are determined from the
        // short name of the enumerated property and the short name of the
        // property value.

        (GENERAL_CATEGORY_CONTROL_V1, "gc=Cc"),
        (GENERAL_CATEGORY_FORMAT_V1, "gc=Cf"),
        (GENERAL_CATEGORY_UNASSIGNED_V1, "gc=Cn"),
        (GENERAL_CATEGORY_PRIVATE_USE_V1, "gc=Co"),
        (GENERAL_CATEGORY_SURROGATE_V1, "gc=S"),
        (GENERAL_CATEGORY_LOWERCASE_LETTER_V1, "gc=Ll"),
        (GENERAL_CATEGORY_MODIFIER_LETTER_V1, "gc=Lm"),
        (GENERAL_CATEGORY_OTHER_LETTER_V1, "gc=Lo"),
        (GENERAL_CATEGORY_TITLECASE_LETTER_V1, "gc=Lt"),
        (GENERAL_CATEGORY_UPPERCASE_LETTER_V1, "gc=Lu"),
        (GENERAL_CATEGORY_SPACING_MARK_V1, "gc=Mc"),
        (GENERAL_CATEGORY_ENCLOSING_MARK_V1, "gc=Me"),
        (GENERAL_CATEGORY_NONSPACING_MARK_V1, "gc=Mn"),
        (GENERAL_CATEGORY_DIGIT_V1, "gc=Nd"),
        (GENERAL_CATEGORY_LETTER_NUMBER_V1, "gc=Nl"),
        (GENERAL_CATEGORY_OTHER_NUMBER_V1, "gc=No"),
        (GENERAL_CATEGORY_CONNECTOR_PUNCTUATION_V1, "gc=Pc"),
        (GENERAL_CATEGORY_DASH_PUNCTUATION_V1, "gc=Pd"),
        (GENERAL_CATEGORY_CLOSE_PUNCTUATION_V1, "gc=Pe"),
        (GENERAL_CATEGORY_FINAL_PUNCTUATION_V1, "gc=Pf"),
        (GENERAL_CATEGORY_INITIAL_PUNCTUATION_V1, "gc=Pi"),
        (GENERAL_CATEGORY_OTHER_PUNCTUATION_V1, "gc=Po"),
        (GENERAL_CATEGORY_OPEN_PUNCTUATION_V1, "gc=Ps"),
        (GENERAL_CATEGORY_CURRENCY_SYMBOL_V1, "gc=Sc"),
        (GENERAL_CATEGORY_MODIFIER_SYMBOL_V1, "gc=Sk"),
        (GENERAL_CATEGORY_MATH_SYMBOL_V1, "gc=Sm"),
        (GENERAL_CATEGORY_OTHER_SYMBOL_V1, "gc=So"),
        (GENERAL_CATEGORY_LINE_SEPARATOR_V1, "gc=Zl"),
        (GENERAL_CATEGORY_PARAGRAPH_SEPARATOR_V1, "gc=Zp"),
        (GENERAL_CATEGORY_SPACE_SEPARATOR_V1, "gc=Zs"),
    );
}

#[icu_provider::data_struct]
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "provider_serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnicodePropertyV1<'s> {
    pub name: Cow<'s, str>,
    pub inv_list: UnicodeSet,
}

impl Default for UnicodePropertyV1<'static> {
    /// Default empty nameless property
    fn default() -> UnicodePropertyV1<'static> {
        UnicodePropertyV1 {
            name: Cow::Borrowed(""),
            inv_list: UnicodeSetBuilder::new().build(),
        }
    }
}

impl<'s> UnicodePropertyV1<'s> {
    pub fn from_uniset(set: &UnicodeSet, name: Cow<'s, str>) -> UnicodePropertyV1<'s> {
        UnicodePropertyV1 { name, inv_list: set.clone() }
    }
}

impl<'s> TryInto<UnicodeSet> for UnicodePropertyV1<'s> {
    type Error = crate::UnicodeSetError;
    fn try_into(self) -> Result<UnicodeSet, Self::Error> {
        Ok(self.inv_list)
    }
}
