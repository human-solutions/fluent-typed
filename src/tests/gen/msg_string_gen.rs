use crate::BundleMessageExt;
#[allow(unused_imports)]
use fluent_bundle::{types::FluentNumber, FluentArgs, FluentBundle, FluentResource, FluentValue};
use std::borrow::Cow;

pub trait MyExt {
    /// $name (String) - The name.
    fn greeting<F0: AsRef<str>>(&self, name: F0) -> Cow<'_, str>;
}

impl MyExt for FluentBundle<FluentResource> {
    fn greeting<F0: AsRef<str>>(&self, name: F0) -> Cow<'_, str> {
        let mut args = FluentArgs::new();
        args.set("name", name.as_ref());
        self.msg_with_args("greeting", args).unwrap()
    }
}
