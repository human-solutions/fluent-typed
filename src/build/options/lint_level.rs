/// How strictly fluent-typed checks the comments in your `.ftl` files.
///
/// fluent-typed infers argument types from message comments
/// (`# $name (String) - ...`). Mistakes in those comments — a typo'd keyword,
/// a misnamed variable, a comment detached from its message — otherwise fail
/// silently. The lint level controls what happens when one is found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LintLevel {
    /// No lint diagnostics are emitted at all.
    Off,
    /// Lint problems are reported as `cargo::warning=` lines. The default.
    #[default]
    Warn,
    /// Comment mistakes in the default locale become hard build errors, but an
    /// untyped variable is still allowed. Use this to enforce correct comments
    /// without also requiring every variable to be type-annotated.
    ///
    /// Diagnostics about non-default locales stay warnings: they concern
    /// translator-owned files and must never block a build.
    Deny,
    /// Like [`LintLevel::Deny`], and additionally every variable of every
    /// generated message must resolve to a concrete type (`String` or
    /// `Number`) — an untyped variable fails the build.
    Strict,
}
