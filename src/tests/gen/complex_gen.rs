use crate::BundleMessageExt;
#[allow(unused_imports)]
use fluent_bundle::{types::FluentNumber, FluentArgs, FluentBundle, FluentResource, FluentValue};
use std::borrow::Cow;

pub trait MyExt {
    fn key<'a, F0: Into<FluentValue<'a>>>(&self, var: F0) -> Cow<'_, str>;
}

impl MyExt for FluentBundle<FluentResource> {
    fn key<'a, F0: Into<FluentValue<'a>>>(&self, var: F0) -> Cow<'_, str> {
        let mut args = FluentArgs::new();
        args.set("var", var);
        self.msg_with_args("key", args).unwrap()
    }
}
