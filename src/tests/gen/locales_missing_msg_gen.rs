// This file is generated. Do not edit it manually.
#![allow(unused_imports)]
use fluent_bundle::{types::FluentNumber, FluentArgs, FluentValue};
use std::borrow::Cow;

use crate::L10nLanguage;

pub struct L10nResources {
    pub profile: String,
    pub company: String,
    pub profile: String,
    pub company: String,
}

impl L10nResources {
    pub fn to_vec(self) -> Vec<String> {
        let mut vec = Vec::new();
        vec.push(self.profile);
        vec.push(self.company);
        vec.push(self.profile);
        vec.push(self.company);
        vec
    }
}

pub struct L10n(L10nLanguage);

impl L10n {
    pub fn load(lang: &str, resources: L10nResources) -> Result<Self, String> {
        Ok(Self(L10nLanguage::new(lang, resources.to_vec())?))
    }

    fn profile_last_name(&self) -> Cow<'_, str> {
        self.0.msg("last-name", None).unwrap()
    }
    fn company_company_name(&self) -> Cow<'_, str> {
        self.0.msg("company-name", None).unwrap()
    }
}
