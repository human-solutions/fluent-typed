mod msg_select;
mod msg_string;
mod msg_text;
mod msg_with_attrib;
mod msg_with_var;

use fluent::{bundle::FluentBundle, FluentResource};
use intl_memoizer::IntlLangMemoizer;
use unic_langid::langid;

#[test]
fn load_bundle() {
    let ftl_string = "hello-world = Hello, world!";
    let bundle = bundle(ftl_string);

    let msg = bundle
        .get_message("hello-world")
        .expect("Message doesn't exist.");
    let mut errors = vec![];
    let pattern = msg.value().expect("Message has no value.");
    let value = bundle.format_pattern(pattern, None, &mut errors);

    assert_eq!(&value, "Hello, world!");
}

fn bundle(ftl: &str) -> FluentBundle<FluentResource, IntlLangMemoizer> {
    let res = FluentResource::try_new(ftl.to_string()).expect("Failed to parse an FTL string.");

    let langid_en = langid!("en-US");
    let mut bundle = FluentBundle::new(vec![langid_en]);
    bundle.set_use_isolating(false);

    bundle
        .add_resource(res)
        .expect("Failed to add FTL resources to the bundle.");
    bundle
}
