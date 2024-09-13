use std::ops::Range;

use unic_langid::LanguageIdentifier;

use crate::prelude::LanguageBundle;

pub struct LanguageCollection {
    langs: Vec<LanguageBundle>,
}

impl LanguageCollection {
    pub fn load<'a, I>(bytes: &[u8], iter: I) -> Result<Self, String>
    where
        I: Iterator<Item = (&'a str, Range<usize>)>,
    {
        Ok(Self {
            langs: iter
                .map(|(lang, range)| LanguageBundle::new(lang, &bytes[range]))
                .collect::<Result<Vec<_>, String>>()?,
        })
    }

    pub fn get(&self, lang: &LanguageIdentifier) -> &LanguageBundle {
        self.langs.iter().find(|b| b.lang() == lang).unwrap()
    }
}
