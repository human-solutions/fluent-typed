// This file is generated. Do not edit it manually.
use crate::prelude::*;
use std::borrow::Cow;

pub struct L10nResources {
    pub base: String,
}

impl L10nResources {
    pub fn to_vec(self) -> Vec<String> {
        let mut vec = Vec::new();
        vec.push(self.base);
        vec
    }
}

pub struct L10n(L10nLanguage);

impl L10n {
    pub fn load(lang: &str, resources: L10nResources) -> Result<Self, String> {
        Ok(Self(L10nLanguage::new(lang, resources.to_vec())?))
    }

    fn cookie_disclaimer_hello_world(&self) -> Cow<'_, str> {
        self.0.msg("hello-world", None).unwrap()
    }
}
