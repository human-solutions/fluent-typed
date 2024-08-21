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

    fn key<'a, F0: Into<FluentValue<'a>>>(&self, var: F0) -> Cow<'_, str> {
        let mut args = FluentArgs::new();
        args.set("var", var);
        self.0.msg("key", Some(args)).unwrap()
    }
}
