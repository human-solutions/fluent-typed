use crate::BundleMessageExt;
#[allow(unused_imports)]
use fluent_bundle::{types::FluentNumber, FluentArgs, FluentBundle, FluentResource, FluentValue};
use std::borrow::Cow;

pub trait MyExt {

    fn settings_twenty_four_hour(&self) -> Cow<'_, str>;

    fn hello_greeting(&self) -> Cow<'_, str>;
}

impl MyExt for FluentBundle<FluentResource> {
    fn settings_twenty_four_hour(&self) -> Cow<'_, str> {
        self.msg("twenty-four-hour", None).unwrap()
    }
    fn hello_greeting(&self) -> Cow<'_, str> {
        self.msg("greeting", None).unwrap()
    }
}
