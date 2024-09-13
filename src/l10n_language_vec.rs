use std::ops::Range;

use unic_langid::LanguageIdentifier;

use crate::prelude::L10nBundle;

pub struct L10nLanguageVec {
    langs: Vec<L10nBundle>,
}

impl L10nLanguageVec {
    pub fn load<'a, I>(bytes: &[u8], iter: I) -> Result<Self, String>
    where
        I: Iterator<Item = (&'a str, Range<usize>)>,
    {
        Ok(Self {
            langs: iter
                .map(|(lang, range)| L10nBundle::new(lang, &bytes[range]))
                .collect::<Result<Vec<_>, String>>()?,
        })
    }

    pub fn get(&self, lang: &LanguageIdentifier) -> &L10nBundle {
        self.langs.iter().find(|b| b.lang() == lang).unwrap()
    }
}
