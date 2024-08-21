// This file is generated. Do not edit it manually.
#![allow(unused_imports)]
use fluent_bundle::{types::FluentNumber, FluentArgs, FluentValue};
use std::borrow::Cow;

use crate::L10nLanguage;

pub struct L10nResources {
    // <<resource definitions>>
}

impl L10nResources {
    pub fn to_vec(self) -> Vec<String> {
        let mut vec = Vec::new();
        // <<resource decomposition>>
        vec
    }
}

pub struct L10n(L10nLanguage);

impl L10n {
    pub fn load(lang: &str, resources: L10nResources) -> Result<Self, String> {
        Ok(Self(L10nLanguage::new(lang, resources.to_vec())?))
    }

    // <<message implementations>>
}
