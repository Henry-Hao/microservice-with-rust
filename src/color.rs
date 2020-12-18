use std::fmt;
use std::str::FromStr;
use std::num::ParseIntError;
use serde::{de::{self, Visitor}, Serialize, Serializer, Deserialize, Deserializer};
use failure::Fail;


#[derive(Clone, PartialEq, Eq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8
}

pub const WHITE: Color = Color { red: 0xFF, green: 0xFF, blue: 0xFF };
pub const BLACK: Color = Color { red: 0x00, green: 0x00, blue: 0x00 };

#[derive(Debug, Fail)]
pub enum ColorError {
    #[fail(display = "invalid value: {}", _0)]
    InvalidValue(String),
    #[fail(display = "parse color's component error: {}", _0)]
    InvalidComponent(#[cause] ParseIntError),
}


impl From<ParseIntError> for ColorError {
    fn from(err: ParseIntError) -> Self {
        ColorError::InvalidComponent(err)
    }
}


impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            &WHITE => write!(f, "White"),
            &BLACK => write!(f, "Black"),
            _ => write!(f, "#{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
        }
    }
}


impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl FromStr for Color {
    type Err = ColorError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "White" => Ok(WHITE.to_owned()),
            "Black" => Ok(BLACK.to_owned()),
            s if s.starts_with("#") && s.len() == 7 => {
                let red = u8::from_str_radix(&s[1..3], 16)?;
                let green = u8::from_str_radix(&s[3..5], 16)?;
                let blue = u8::from_str_radix(&s[5..7], 16)?;
                Ok(Color{red, green, blue})
            },
            _ => {
                Err(ColorError::InvalidValue(s.to_string()))
            }
        }
    }
}


struct ColorVisitor;

impl <'de>Visitor<'de> for ColorVisitor {
    type Value = Color;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "expecting a color value")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where E: de::Error, {
        v.parse::<Color>().map_err(de::Error::custom)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where E: de::Error, {
        self.visit_str(v.as_ref())
    }

}

impl<'a> Deserialize<'a> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        deserializer.deserialize_any(ColorVisitor)
    }
}

