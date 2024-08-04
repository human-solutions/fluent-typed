use crate::BundleMessageExt;
#[allow(unused_imports)]
use fluent_bundle::{types::FluentNumber, FluentArgs, FluentBundle, FluentResource, FluentValue};
use std::borrow::Cow;

pub trait MyExt {
    /// $duration (Number) - The duration in seconds.

    fn time_elapsed<F0: Into<FluentNumber>>(&self, duration: F0) -> Cow<'_, str>;
}

impl MyExt for FluentBundle<FluentResource> {
    fn time_elapsed<F0: Into<FluentNumber>>(&self, duration: F0) -> Cow<'_, str> {
        let mut args = FluentArgs::new();
        args.set("duration", duration.into());
        self.msg("time-elapsed", Some(args)).unwrap()
    }
}
