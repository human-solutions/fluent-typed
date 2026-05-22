use std::ops::Range;

use crate::prelude::L10nBundle;

pub struct L10nLanguageVec {
    langs: Vec<L10nBundle>,
}

impl L10nLanguageVec {
    /// Load all languages, with Unicode bidi isolation marks around
    /// interpolated variables. See [`L10nBundle::new`].
    pub fn load<S, I>(bytes: &[u8], iter: I) -> Result<Self, String>
    where
        S: AsRef<str>,
        I: Iterator<Item = (S, Range<usize>)>,
    {
        Self::build(bytes, iter, true)
    }

    /// Load all languages, without Unicode bidi isolation marks. See
    /// [`L10nBundle::new_without_isolation`].
    pub fn load_without_isolation<S, I>(bytes: &[u8], iter: I) -> Result<Self, String>
    where
        S: AsRef<str>,
        I: Iterator<Item = (S, Range<usize>)>,
    {
        Self::build(bytes, iter, false)
    }

    fn build<S, I>(bytes: &[u8], iter: I, use_isolating: bool) -> Result<Self, String>
    where
        S: AsRef<str>,
        I: Iterator<Item = (S, Range<usize>)>,
    {
        Ok(Self {
            langs: iter
                .map(|(lang, range)| {
                    let data = &bytes[range];
                    if use_isolating {
                        L10nBundle::new(lang, data)
                    } else {
                        L10nBundle::new_without_isolation(lang, data)
                    }
                })
                .collect::<Result<Vec<_>, String>>()?,
        })
    }

    /// IMPORTANT, the lang argument should be a L10n enum variant
    pub fn get(&self, lang: impl AsRef<str>) -> &L10nBundle {
        self.langs
            .iter()
            .find(|b| b.lang() == lang.as_ref())
            .unwrap()
    }
}
