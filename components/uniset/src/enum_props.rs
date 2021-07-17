// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

#![allow(clippy::upper_case_acronyms)]

/// Selection constants for Unicode properties.
/// These constants are used to select one of the Unicode properties.
/// See UProperty in ICU4C.
#[derive(Clone, PartialEq, Debug)]
pub enum EnumeratedProperty {
    GeneralCategory = 0x1005,
    Script = 0x100A,
    ScriptExtensions = 0x7000,
}

/// Enumerated Unicode general category types.
/// See https://www.unicode.org/reports/tr44/ .
/// See UCharCategory in ICU4C.
#[derive(Clone, PartialEq, Debug)]
pub enum GeneralCategory {
    Control = 15,
    Format = 16,
    Unassigned = 0,
    PrivateUse = 17,
    Surrogate = 18,
    LowercaseLetter = 2,
    ModifierLetter = 4,
    OtherLetter = 5,
    TitlecaseLetter = 3,
    UppercaseLetter = 1,
    SpacingMark = 8,
    EnclosingMark = 7,
    NonspacingMark = 6,
    Digit = 9,
    LetterNumber = 10,
    OtherNumber = 11,
    ConnectorPunctuation = 22,
    DashPunctuation = 19,
    ClosePunctuation = 21,
    FinalPunctuation = 29,
    InitialPunctuation = 28,
    OtherPunctuation = 23,
    OpenPunctuation = 20,
    CurrencySymbol = 25,
    ModifierSymbol = 26,
    MathSymbol = 24,
    OtherSymbol = 27,
    LineSeparator = 13,
    ParagraphSeparator = 14,
    SpaceSeparator = 12,
}

//// Enumerated property Script.
///
/// For more information, see UAX #24: http://www.unicode.org/reports/tr24/.
/// See UScriptCode in ICU4C.
#[derive(Clone, PartialEq, Debug)]
pub enum Script {
    Adlam = 167,
    Afaka = 147,
    Ahom = 161,
    AnatolianHieroglyphs = 156,
    Arabic = 2,
    Armenian = 3,
    Avestan = 117,
    Balinese = 62,
    Bamum = 130,
    BassaVah = 134,
    Batak = 63,
    Bengali = 4,
    Bhaiksuki = 168,
    Blissymbols = 64,
    BookPahlavi = 124,
    Bopomofo = 5,
    Brahmi = 65,
    Braille = 46,
    Buginese = 55,
    Buhid = 44,
    CanadianAboriginal = 40,
    Carian = 104,
    CaucasianAlbanian = 159,
    Chakma = 118,
    Cham = 66,
    Cherokee = 6,
    Chorasmian = 189,
    Cirth = 67,
    CodeLimit = 193,
    Common = 0,
    Coptic = 7,
    Cuneiform = 101,
    Cypriot = 47,
    Cyrillic = 8,
    DemoticEgyptian = 69,
    Deseret = 9,
    Devanagari = 10,
    DivesAkuru = 190,
    Dogra = 178,
    Duployan = 135,
    EasternSyriac = 97,
    EgyptianHieroglyphs = 71,
    Elbasan = 136,
    Elymaic = 185,
    EstrangeloSyriac = 95,
    Ethiopic = 11,
    Georgian = 12,
    Glagolitic = 56,
    Gothic = 13,
    Grantha = 137,
    Greek = 14,
    Gujarati = 15,
    GunjalaGondi = 179,
    Gurmukhi = 16,
    Han = 17,
    HanWithBopomofo = 172,
    Hangul = 18,
    HanifiRohingya = 182,
    Hanunoo = 43,
    HarappanIndus = 77,
    Hatran = 162,
    Hebrew = 19,
    HieraticEgyptian = 70,
    Hiragana = 20,
    ImperialAramaic = 116,
    Inherited = 1,
    InscriptionalPahlavi = 122,
    InscriptionalParthian = 125,
    InvalidCode = -1,
    Jamo = 173,
    Japanese = 105,
    Javanese = 78,
    Jurchen = 148,
    Kaithi = 120,
    Kannada = 21,
    Katakana = 22,
    KatakanaOrHiragana = 54,
    KayahLi = 79,
    Kharoshthi = 57,
    KhitanSmallScript = 191,
    Khmer = 23,
    Khojki = 157,
    Khudawadi = 145,
    Khutsuri = 72,
    Korean = 119,
    Kpelle = 138,
    Lanna = 106,
    Lao = 24,
    Latin = 25,
    LatinFraktur = 80,
    LatinGaelic = 81,
    Lepcha = 82,
    Limbu = 48,
    LinearA = 83,
    LinearB = 49,
    Lisu = 131,
    Loma = 139,
    Lycian = 107,
    Lydian = 108,
    Mahajani = 160,
    Makasar = 180,
    Malayalam = 26,
    Mandaic = 84,
    Manichaean = 121,
    Marchen = 169,
    MasaramGondi = 175,
    MathematicalNotation = 128,
    MayanHieroglyphs = 85,
    Medefaidrin = 181,
    MeiteiMayek = 115,
    Mende = 140,
    MeroiticCursive = 141,
    MeroiticHieroglyphs = 86,
    Miao = 92,
    Modi = 163,
    Mongolian = 27,
    Moon = 114,
    Mro = 149,
    Multani = 164,
    Myanmar = 28,
    Nabataean = 143,
    NakhiGeba = 132,
    Nandinagari = 187,
    NewTaiLue = 59,
    Newa = 170,
    Nko = 87,
    Nushu = 150,
    NyiakengPuachueHmong = 186,
    Ogham = 29,
    OlChiki = 109,
    OldChurchSlavonicCyrillic = 68,
    OldHungarian = 76,
    OldItalic = 30,
    OldNorthArabian = 142,
    OldPermic = 89,
    OldPersian = 61,
    OldSogdian = 184,
    OldSouthArabian = 133,
    Oriya = 31,
    Orkhon = 88,
    Osage = 171,
    Osmanya = 50,
    PahawhHmong = 75,
    Palmyrene = 144,
    PauCinHau = 165,
    PhagsPa = 90,
    Phoenician = 91,
    PsalterPahlavi = 123,
    Rejang = 110,
    Rongorongo = 93,
    Runic = 32,
    Samaritan = 126,
    Sarati = 94,
    Saurashtra = 111,
    Sharada = 151,
    Shavian = 51,
    Siddham = 166,
    SignWriting = 112,
    SimplifiedHan = 73,
    Sinhala = 33,
    Sogdian = 183,
    SoraSompeng = 152,
    Soyombo = 176,
    Sundanese = 113,
    SylotiNagri = 58,
    Symbols = 129,
    SymbolsEmoji = 174,
    Syriac = 34,
    Tagalog = 42,
    Tagbanwa = 45,
    TaiLe = 52,
    TaiViet = 127,
    Takri = 153,
    Tamil = 35,
    Tangut = 154,
    Telugu = 36,
    Tengwar = 98,
    Thaana = 37,
    Thai = 38,
    Tibetan = 39,
    Tifinagh = 60,
    Tirhuta = 158,
    TraditionalHan = 74,
    Ugaritic = 53,
    Unknown = 103,
    UnwrittenLanguages = 102,
    Vai = 99,
    VisibleSpeech = 100,
    Wancho = 188,
    WarangCiti = 146,
    WesternSyriac = 96,
    Woleai = 155,
    Yezidi = 192,
    Yi = 41,
    ZanabazarSquare = 177,
}
