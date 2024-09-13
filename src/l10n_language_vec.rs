use std::ops::Range;

use crate::prelude::L10nBundle;

pub struct L10nLanguageVec {
    langs: Vec<L10nBundle>,
}

impl L10nLanguageVec {
    pub fn load<S, I>(bytes: &[u8], iter: I) -> Result<Self, String>
    where
        S: AsRef<str>,
        I: Iterator<Item = (S, Range<usize>)>,
    {
        Ok(Self {
            langs: iter
                .map(|(lang, range)| L10nBundle::new(lang, &bytes[range]))
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
