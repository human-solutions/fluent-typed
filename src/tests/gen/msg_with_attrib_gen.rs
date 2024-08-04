use crate::BundleMessageExt;
#[allow(unused_imports)]
use fluent_bundle::{types::FluentNumber, FluentArgs, FluentBundle, FluentResource, FluentValue};
use std::borrow::Cow;

pub trait MyExt {
    /// This is a message comment

    fn hello(&self) -> Cow<'_, str>;
    fn hello_tooltip<'a, F0: Into<FluentValue<'a>>>(&self, user_name: F0) -> Cow<'_, str>;
}

impl MyExt for FluentBundle<FluentResource> {
    fn hello(&self) -> Cow<'_, str> {
        self.msg("hello", None).unwrap()
    }
    fn hello_tooltip<'a, F0: Into<FluentValue<'a>>>(&self, user_name: F0) -> Cow<'_, str> {
        let mut args = FluentArgs::new();
        args.set("userName", user_name);
        self.attr("hello", "tooltip", Some(args)).unwrap()
    }
}
