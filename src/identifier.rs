use std::fmt::{Debug, Display, Write};

static mut GLOBAL_STRINGS: Vec<String> = Vec::new(); // used for identifiers (things that persist throughout the ENTIRE game); Yes, this is unsafe and not thread safe

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct GlobalString(usize);

impl From<&str> for GlobalString {
    fn from(value: &str) -> Self {
        for (id, str) in unsafe { GLOBAL_STRINGS.iter().enumerate() } {
            if str == value {
                return GlobalString(id);
            }
        }
        unsafe {
            GLOBAL_STRINGS.push(value.to_string());
            GlobalString(GLOBAL_STRINGS.len() - 1)
        }
    }
}

impl From<&String> for GlobalString {
    fn from(value: &String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<String> for GlobalString {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl Default for GlobalString {
    fn default() -> Self {
        Self::from("")
    }
}

impl GlobalString {
    pub fn as_str(&self) -> &'static String {
        unsafe {
            &GLOBAL_STRINGS[self.0]
        }
    }

    /// Gets the id from GlobalString; This **isn't** recommended as there are not a whole lot of areas where you'd want this
    /// Safety: This is a 100% safe operation but marked as unsafe, as this should be avoided at all cost
    pub unsafe fn get_id(&self) -> usize {
        self.0
    }

    /// Gets a GlobalString from a raw id; This **isn't** recommended and you should use GlobalString::from
    /// Safety: You have to make sure that id is a correct id
    pub unsafe fn from_raw(id: usize) -> Self {
        Self(id) // usually u wanna put this in its own inlined and safe function as this operation isnt unsafe, just doing it isnt recommended
    }
}

impl Debug for GlobalString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("GlobalString#{}:{}", self.0, self.as_str()))
    }
}

impl Display for GlobalString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.as_str())
    }
}


#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Identifier {
    major: GlobalString,
    minor: GlobalString,
}

impl Debug for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.major.as_str())?;
        f.write_char(':')?;
        f.write_str(&self.minor.as_str())
    }
}

impl From<(GlobalString, GlobalString)> for Identifier {
    fn from((major, minor): (GlobalString, GlobalString)) -> Self {
        Self { major, minor }
    }
}

impl From<(String, String)> for Identifier {
    fn from((major, minor): (String, String)) -> Self {
        Self { major: GlobalString::from(major), minor: GlobalString::from(minor) }
    }
}

impl From<&(String, String)> for Identifier {
    fn from((major, minor): &(String, String)) -> Self {
        Self { major: GlobalString::from(major), minor: GlobalString::from(minor) }
    }
}

impl From<(&str, &str)> for Identifier {
    fn from((major, minor): (&str, &str)) -> Self {
        Self { major: GlobalString::from(major), minor: GlobalString::from(minor) }
    }
}

impl From<&(&str, &str)> for Identifier {
    fn from(&(major, minor): &(&str, &str)) -> Self {
        Self { major: GlobalString::from(major), minor: GlobalString::from(minor) }
    }
}