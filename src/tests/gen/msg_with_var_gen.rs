// This file is generated. Do not edit it manually.
#![allow(unused_imports)]
use fluent_bundle::{types::FluentNumber, FluentArgs, FluentValue};
use std::borrow::Cow;

use crate::L10nLanguage;

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

    fn hello<'a, F0: Into<FluentValue<'a>>>(&self, first_name: F0) -> Cow<'_, str> {
        let mut args = FluentArgs::new();
        args.set("first-name", first_name);
        self.0.msg("hello", Some(args)).unwrap()
    }
}
