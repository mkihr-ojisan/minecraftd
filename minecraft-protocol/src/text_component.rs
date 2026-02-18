use std::io::Write;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TextComponent {
    String(String),
    Object(Object),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Object {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underlined: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strikethrough: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obfuscated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Vec<TextComponent>>,
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Hex(u32),
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
}

impl Color {
    fn to_rgb(self) -> u32 {
        match self {
            Color::Hex(hex) => hex,
            Color::Black => 0x000000,
            Color::DarkBlue => 0x0000AA,
            Color::DarkGreen => 0x00AA00,
            Color::DarkAqua => 0x00AAAA,
            Color::DarkRed => 0xAA0000,
            Color::DarkPurple => 0xAA00AA,
            Color::Gold => 0xFFAA00,
            Color::Gray => 0xAAAAAA,
            Color::DarkGray => 0x555555,
            Color::Blue => 0x5555FF,
            Color::Green => 0x55FF55,
            Color::Aqua => 0x55FFFF,
            Color::Red => 0xFF5555,
            Color::LightPurple => 0xFF55FF,
            Color::Yellow => 0xFFFF55,
            Color::White => 0xFFFFFF,
        }
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Color::Hex(hex) => format!("#{:06x}", hex).serialize(serializer),
            Color::Black => "black".serialize(serializer),
            Color::DarkBlue => "dark_blue".serialize(serializer),
            Color::DarkGreen => "dark_green".serialize(serializer),
            Color::DarkAqua => "dark_aqua".serialize(serializer),
            Color::DarkRed => "dark_red".serialize(serializer),
            Color::DarkPurple => "dark_purple".serialize(serializer),
            Color::Gold => "gold".serialize(serializer),
            Color::Gray => "gray".serialize(serializer),
            Color::DarkGray => "dark_gray".serialize(serializer),
            Color::Blue => "blue".serialize(serializer),
            Color::Green => "green".serialize(serializer),
            Color::Aqua => "aqua".serialize(serializer),
            Color::Red => "red".serialize(serializer),
            Color::LightPurple => "light_purple".serialize(serializer),
            Color::Yellow => "yellow".serialize(serializer),
            Color::White => "white".serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match String::deserialize(deserializer)?.as_str() {
            "black" => Ok(Color::Black),
            "dark_blue" => Ok(Color::DarkBlue),
            "dark_green" => Ok(Color::DarkGreen),
            "dark_aqua" => Ok(Color::DarkAqua),
            "dark_red" => Ok(Color::DarkRed),
            "dark_purple" => Ok(Color::DarkPurple),
            "gold" => Ok(Color::Gold),
            "gray" => Ok(Color::Gray),
            "dark_gray" => Ok(Color::DarkGray),
            "blue" => Ok(Color::Blue),
            "green" => Ok(Color::Green),
            "aqua" => Ok(Color::Aqua),
            "red" => Ok(Color::Red),
            "light_purple" => Ok(Color::LightPurple),
            "yellow" => Ok(Color::Yellow),
            "white" => Ok(Color::White),
            s => {
                if !s.starts_with('#') || s.len() != 7 {
                    return Err(serde::de::Error::custom("Invalid color"));
                }

                let hex = u32::from_str_radix(&s[1..], 16).map_err(serde::de::Error::custom)?;
                Ok(Color::Hex(hex))
            }
        }
    }
}

impl TextComponent {
    pub fn print(&self, w: &mut impl Write) -> std::io::Result<()> {
        match self {
            TextComponent::String(str) => write!(w, "{}", str),
            TextComponent::Object(obj) => {
                write!(w, "{}", obj.text.as_deref().unwrap_or_default())?;
                for extra in obj.extra.iter().flatten() {
                    extra.print(w)?;
                }
                Ok(())
            }
        }
    }

    pub fn print_formatted(&self, w: &mut impl Write) -> std::io::Result<()> {
        self.internal_print_formatted(w, None, false, false, false, false, false)
    }

    #[allow(clippy::too_many_arguments)]
    fn internal_print_formatted(
        &self,
        w: &mut impl Write,
        mut color: Option<Color>,
        mut bold: bool,
        mut italic: bool,
        mut underlined: bool,
        mut strikethrough: bool,
        mut obfuscated: bool,
    ) -> std::io::Result<()> {
        if let TextComponent::Object(obj) = self {
            if let Some(c) = obj.color {
                color = Some(c);
            }
            if let Some(b) = obj.bold {
                bold = b;
            }
            if let Some(i) = obj.italic {
                italic = i;
            }
            if let Some(u) = obj.underlined {
                underlined = u;
            }
            if let Some(s) = obj.strikethrough {
                strikethrough = s;
            }
            if let Some(o) = obj.obfuscated {
                obfuscated = o;
            }
        }

        let str = match self {
            TextComponent::String(str) => str,
            TextComponent::Object(obj) => obj.text.as_deref().unwrap_or_default(),
        };
        let mut char_indices = str.char_indices().peekable();

        let mut start = 0;
        while let Some((i, c)) = char_indices.next() {
            if c == '§' {
                if start < i {
                    Self::internal_print_formatted_str(
                        &str[start..i],
                        w,
                        color,
                        bold,
                        italic,
                        underlined,
                        strikethrough,
                        obfuscated,
                    )?;
                }

                let Some((_, format)) = char_indices.next() else {
                    break;
                };
                match format {
                    '0' => color = Some(Color::Black),
                    '1' => color = Some(Color::DarkBlue),
                    '2' => color = Some(Color::DarkGreen),
                    '3' => color = Some(Color::DarkAqua),
                    '4' => color = Some(Color::DarkRed),
                    '5' => color = Some(Color::DarkPurple),
                    '6' => color = Some(Color::Gold),
                    '7' => color = Some(Color::Gray),
                    '8' => color = Some(Color::DarkGray),
                    '9' => color = Some(Color::Blue),
                    'a' => color = Some(Color::Green),
                    'b' => color = Some(Color::Aqua),
                    'c' => color = Some(Color::Red),
                    'd' => color = Some(Color::LightPurple),
                    'e' => color = Some(Color::Yellow),
                    'f' => color = Some(Color::White),
                    'k' => obfuscated = true,
                    'l' => bold = true,
                    'm' => strikethrough = true,
                    'n' => underlined = true,
                    'o' => italic = true,
                    'r' => {
                        color = None;
                        bold = false;
                        italic = false;
                        underlined = false;
                        strikethrough = false;
                        obfuscated = false;
                    }
                    _ => {}
                }

                start = if let Some((next, _)) = char_indices.peek() {
                    *next
                } else {
                    str.len()
                };
            }
        }
        if start < str.len() {
            Self::internal_print_formatted_str(
                &str[start..],
                w,
                color,
                bold,
                italic,
                underlined,
                strikethrough,
                obfuscated,
            )?;
        }

        if let TextComponent::Object(obj) = self {
            for extra in obj.extra.iter().flatten() {
                extra.internal_print_formatted(
                    w,
                    color,
                    bold,
                    italic,
                    underlined,
                    strikethrough,
                    obfuscated,
                )?;
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn internal_print_formatted_str(
        str: &str,
        w: &mut impl Write,
        color: Option<Color>,
        bold: bool,
        italic: bool,
        underlined: bool,
        strikethrough: bool,
        obfuscated: bool,
    ) -> std::io::Result<()> {
        if let Some(color) = color {
            let rgb = color.to_rgb();
            let r = (rgb >> 16) & 0xFF;
            let g = (rgb >> 8) & 0xFF;
            let b = rgb & 0xFF;
            write!(w, "\x1b[38;2;{r};{g};{b}m")?;
        }

        if bold {
            write!(w, "\x1b[1m")?;
        }

        if italic {
            write!(w, "\x1b[3m")?;
        }

        if underlined {
            write!(w, "\x1b[4m")?;
        }

        if strikethrough {
            write!(w, "\x1b[9m")?;
        }

        if obfuscated {
            write!(w, "\x1b[8m")?;
        }

        write!(w, "{}\x1b[0m", str)?;

        Ok(())
    }
}
