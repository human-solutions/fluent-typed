# Structured messages and bidi

## Structured messages

Some translations need a UI element in the middle of a sentence: an icon, a link, a
button. Splitting the sentence into separate messages breaks translations, because
word order differs between languages. Instead, mark the variable or term with
`(Element)`, and fluent-typed generates a struct of text pieces split at those
points:

```ftl
# $count (Number) - How many unread.
# $icon (Element) - An icon injected by the app.
# -privacy-link (Element) - Link text the app wraps in a link.
notice = { $icon } You have { $count } unread, see { -privacy-link }.
-privacy-link = our privacy policy
```

There are two kinds of element:

- an `(Element)` **variable** (`$icon`) is an empty slot: the app puts its own element
  there;
- an `(Element)` **term** (`-privacy-link`) is translated text that the app wraps in an
  element.

The accessor returns a struct with the pieces in the order they're shown. For the
message above, the generated struct is:

<!-- docs-check: skip -->
```rust
pub struct Notice {
    pub s0: String,           // text before $icon
    pub icon: ElementGap,     // the $icon slot, filled by the app
    pub s1: String,           // text between the elements
    pub privacy_link: String, // the translated -privacy-link text
    pub s2: String,           // text after -privacy-link
}
```

Only real arguments are parameters; the elements are not:

```rust
let n: Notice = strs.msg_notice(3);

// Render it, filling the slot and wrapping the term:
let html = format!(
    r#"<bdi>{}</bdi><img src="mail.svg"><bdi>{}</bdi><a href="/privacy"><bdi>{}</bdi></a><bdi>{}</bdi>"#,
    n.s0, n.s1, n.privacy_link, n.s2,
);

// `Notice` also implements `Display`, which joins the text for plain-text use:
let plain = n.to_string();
```

Selectors and ordinary variables inside the pieces are already resolved, so the app
never has to handle plural rules. Wrap each piece and each injected element in its own
isolated bidi run, as with the `<bdi>` above (or CSS `unicode-bidi: isolate`); see
the next section.

A term and a message can't share a name: `-foo` and `foo` collide inside
fluent-bundle, which stores both as `foo`. The build reports both locations instead of
letting it crash at runtime.

## Bidi isolation

By default, every interpolated variable is wrapped in Unicode bidi isolation marks
(FSI `U+2068` … PDI `U+2069`):

```rust
assert_eq!(strs.msg_hello("Sam"), "Hello \u{2068}Sam\u{2069}");
```

The marks stop an interpolated value's text direction from scrambling the surrounding
sentence. That matters as soon as you support a right-to-left language, such as Arabic
or Hebrew, or interpolate text from users, which can be in any script. Browsers and
other bidi-aware renderers don't display the marks.

If your strings are never shown in a bidi-aware context, and you don't use
right-to-left languages or interpolate user text, you can turn the marks off:

```rust
// build.rs
let options = BuildOptions::default().without_bidi_isolation();
```

Watch out for this in tests and string comparisons: with isolation on, `hello` is
`"Hello \u{2068}Sam\u{2069}"`, not `"Hello Sam"`.
