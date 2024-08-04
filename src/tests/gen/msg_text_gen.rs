use crate::BundleMessageExt;
#[allow(unused_imports)]
use fluent_bundle::{types::FluentNumber, FluentArgs, FluentBundle, FluentResource, FluentValue};
use std::borrow::Cow;

pub trait MyExt {

    fn hello_world(&self) -> Cow<'_, str>;
}

impl MyExt for FluentBundle<FluentResource> {
    fn hello_world(&self) -> Cow<'_, str> {
        self.msg("hello-world", None).unwrap()
    }
}
