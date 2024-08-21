// This file is generated. Do not edit it manually.
#![allow(unused_imports)]
use fluent_bundle::{types::FluentNumber, FluentArgs, FluentValue};
use std::borrow::Cow;

use crate::L10nLanguage;

pub struct L10nResources {
    pub settings: String,
    pub hello: String,
    pub settings: String,
    pub hello: String,
}

impl L10nResources {
    pub fn to_vec(self) -> Vec<String> {
        let mut vec = Vec::new();
        vec.push(self.settings);
        vec.push(self.hello);
        vec.push(self.settings);
        vec.push(self.hello);
        vec
    }
}

pub struct L10n(L10nLanguage);

impl L10n {
    pub fn load(lang: &str, resources: L10nResources) -> Result<Self, String> {
        Ok(Self(L10nLanguage::new(lang, resources.to_vec())?))
    }

    fn settings_twenty_four_hour(&self) -> Cow<'_, str> {
        self.0.msg("twenty-four-hour", None).unwrap()
    }
    fn hello_greeting(&self) -> Cow<'_, str> {
        self.0.msg("greeting", None).unwrap()
    }
}
