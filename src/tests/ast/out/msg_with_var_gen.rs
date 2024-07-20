use crate::BundleMessageExt;
#[allow(unused_imports)]
use fluent_bundle::{types::FluentNumber, FluentArgs, FluentBundle, FluentResource, FluentValue};
use std::borrow::Cow;

pub trait MyExt {
    fn hello<'a, F0: Into<FluentValue<'a>>>(&self, first_name: F0) -> Cow<'_, str>;
}

impl MyExt for FluentBundle<FluentResource> {
    fn hello<'a, F0: Into<FluentValue<'a>>>(&self, first_name: F0) -> Cow<'_, str> {
        let mut args = FluentArgs::new();
        args.set("first-name", first_name);
        self.msg_with_args("hello", args).unwrap()
    }
}
