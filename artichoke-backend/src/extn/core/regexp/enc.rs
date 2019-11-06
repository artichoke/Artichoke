//! Parse encoding parameter to `Regexp#initialize` and `Regexp::compile`.

use std::hash::{Hash, Hasher};

use crate::extn::core::regexp::Regexp;
use crate::types::Int;
use crate::value::{Value, ValueLike};
use crate::Artichoke;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Error {
    InvalidEncoding,
}

#[derive(Debug, Clone, Copy)]
pub enum Encoding {
    Fixed,
    No,
    None,
}

impl Encoding {
    pub fn flags(self) -> Int {
        match self {
            Self::Fixed => Regexp::FIXEDENCODING,
            Self::No => Regexp::NOENCODING,
            Self::None => 0,
        }
    }

    pub fn string(self) -> &'static str {
        match self {
            Self::Fixed | Self::None => "",
            Self::No => "n",
        }
    }
}

impl Default for Encoding {
    fn default() -> Self {
        Self::None
    }
}

impl Hash for Encoding {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.string().hash(state);
    }
}

impl PartialEq for Encoding {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::No, Self::None)
            | (Self::None, Self::No)
            | (Self::No, Self::No)
            | (Self::None, Self::None)
            | (Self::Fixed, Self::Fixed) => true,
            _ => false,
        }
    }
}

impl Eq for Encoding {}

pub fn parse(interp: &Artichoke, value: &Value) -> Result<Encoding, Error> {
    if let Ok(encoding) = value.itself::<Int>(interp) {
        // Only deal with Encoding opts
        let encoding = encoding & !Regexp::ALL_REGEXP_OPTS;
        if encoding == Regexp::FIXEDENCODING {
            Ok(Encoding::Fixed)
        } else if encoding == Regexp::NOENCODING {
            Ok(Encoding::No)
        } else if encoding == 0 {
            Ok(Encoding::default())
        } else {
            Err(Error::InvalidEncoding)
        }
    } else if let Ok(encoding) = value.itself::<&str>(interp) {
        if encoding.contains('u') && encoding.contains('n') {
            return Err(Error::InvalidEncoding);
        }
        let mut enc = vec![];
        for flag in encoding.chars() {
            match flag {
                'u' | 's' | 'e' => enc.push(Encoding::Fixed),
                'n' => enc.push(Encoding::No),
                'i' | 'm' | 'x' | 'o' => continue,
                _ => return Err(Error::InvalidEncoding),
            }
        }
        if enc.len() > 1 {
            return Err(Error::InvalidEncoding);
        }
        Ok(enc.pop().unwrap_or_default())
    } else {
        Ok(Encoding::default())
    }
}
