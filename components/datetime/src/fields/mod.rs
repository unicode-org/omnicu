// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/master/LICENSE ).
mod length;
pub(crate) mod symbols;

pub use length::{FieldLength, LengthError};
pub use symbols::*;

use std::convert::{TryFrom, TryInto};

#[derive(Debug)]
pub enum Error {
    TooLong(FieldSymbol),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Field {
    pub symbol: FieldSymbol,
    pub length: FieldLength,
}

impl Field {
    pub fn get_length_type(&self) -> TextOrNumeric {
        match self.symbol {
            FieldSymbol::Year(year) => year.get_length_type(&self.length),
            FieldSymbol::Month(month) => month.get_length_type(&self.length),
            FieldSymbol::Day(day) => day.get_length_type(&self.length),
            FieldSymbol::Weekday(weekday) => weekday.get_length_type(&self.length),
            FieldSymbol::DayPeriod(day_period) => day_period.get_length_type(&self.length),
            FieldSymbol::Hour(hour) => hour.get_length_type(&self.length),
            FieldSymbol::Minute => TextOrNumeric::Numeric,
            FieldSymbol::Second(second) => second.get_length_type(&self.length),
        }
    }
}

impl From<(FieldSymbol, FieldLength)> for Field {
    fn from(input: (FieldSymbol, FieldLength)) -> Self {
        Self {
            symbol: input.0,
            length: input.1,
        }
    }
}

impl TryFrom<(FieldSymbol, usize)> for Field {
    type Error = Error;
    fn try_from(input: (FieldSymbol, usize)) -> Result<Self, Self::Error> {
        let (symbol, length) = (
            input.0,
            input
                .1
                .try_into()
                .map_err(|_| Self::Error::TooLong(input.0))?,
        );
        Ok(Self { symbol, length })
    }
}
