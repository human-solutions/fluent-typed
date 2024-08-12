use crate::BundleMessageExt;
#[allow(unused_imports)]
use fluent_bundle::{types::FluentNumber, FluentArgs, FluentBundle, FluentResource, FluentValue};
use std::borrow::Cow;

pub trait MyExt {

    fn twelve_hour(&self) -> Cow<'_, str>;

    fn twenty_four_hour(&self) -> Cow<'_, str>;
}

impl MyExt for FluentBundle<FluentResource> {
    fn twelve_hour(&self) -> Cow<'_, str> {
        self.msg("twelve-hour", None).unwrap()
    }
    fn twenty_four_hour(&self) -> Cow<'_, str> {
        self.msg("twenty-four-hour", None).unwrap()
    }
}
