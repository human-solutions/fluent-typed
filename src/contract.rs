//! Runtime validation of external `.ftl` translations.
//!
//! The build step compiles a [`MessageContract`] for every generated accessor
//! into the generated file (the `MESSAGE_CONTRACTS` static). [`validate_ftl`]
//! checks external `.ftl` bytes — a translation read from disk or downloaded
//! at runtime — against those contracts: every generated message and attribute
//! is defined, uses only variables its accessor passes (the same rule the
//! build applies to every locale), keeps its `(Element)` markers, and
//! references only messages and terms that exist (transitively, without
//! cycles), and calls no function but `NUMBER()`, the only one the runtime
//! registers. The build runs the same reference check on every locale.

use std::borrow::Cow;
use std::fmt;

use fluent_syntax::{ast, parser};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::error::L10nError;
use crate::ftl_refs::{Ref, RefKind, RefsIncompat, check_refs, find_refs_and_selectors};

/// The functions the runtime registers: fluent-bundle's builtins. Calling any
/// other function fails to format.
const KNOWN_FUNCTIONS: &[&str] = &["NUMBER"];

/// The compiled contract of one generated message accessor: which message (or
/// attribute) it resolves, which arguments it fills, and — for structured
/// messages — the exact `(Element)` marker sequence it splits on.
///
/// Instances are emitted by the build step into the generated file's
/// `MESSAGE_CONTRACTS` static; there is no reason to construct one by hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageContract {
    /// The Fluent message id.
    pub message: &'static str,
    /// The attribute of the message, for an attribute accessor.
    pub attribute: Option<&'static str>,
    /// The argument names the accessor fills (`$variable` names).
    pub vars: &'static [&'static str],
    /// Variables encoded from Rust `bool` as Fluent `"true"` / `"false"`.
    pub bool_vars: &'static [&'static str],
    /// The `(Element)` markers, in pattern order. Empty for plain messages.
    pub elements: &'static [ElementContract],
}

/// One `(Element)` marker of a structured message contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElementContract {
    /// The variable or term name (without the `$` / `-` sigil).
    pub name: &'static str,
    /// `true` for a term element (`-name`), `false` for a variable element.
    pub is_term: bool,
}

/// One way an external `.ftl` failed to satisfy a [`MessageContract`].
///
/// Carried by [`L10nError::Validation`]. Each variant describes a mistake
/// that would otherwise surface later as a panic or formatting error in a
/// generated accessor.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ContractViolation {
    /// A contract message is missing from the `.ftl` entirely.
    MissingMessage {
        /// The message id.
        message: String,
    },
    /// The message exists but has no value of its own, and the contract
    /// expects one (the message defines only attributes).
    MissingValue {
        /// The message id.
        message: String,
    },
    /// The message exists but lacks a required attribute.
    MissingAttribute {
        /// The message id.
        message: String,
        /// The missing attribute.
        attribute: String,
    },
    /// The pattern references a variable that is not a contract argument.
    /// The generated accessor would leave it unfilled, which fails to format.
    UnknownVariable {
        /// The message id.
        message: String,
        /// The attribute, for an attribute accessor.
        attribute: Option<String>,
        /// The unknown `$variable` name.
        variable: String,
    },
    /// The `(Element)` marker sequence does not match the contract, so the
    /// structured-message split would not produce the expected slots.
    /// The entries are rendered as `$variable` / `-term`.
    ElementMismatch {
        /// The message id.
        message: String,
        /// The contract's marker sequence.
        expected: Vec<String>,
        /// The sequence found in the `.ftl`.
        found: Vec<String>,
    },
    /// A Boolean variable's selector does not contain exactly `[true]` and
    /// `[false]` branches.
    BoolSelectorMismatch {
        /// The message id.
        message: String,
        /// The attribute, for an attribute accessor.
        attribute: Option<String>,
        /// The Boolean variable driving the selector.
        variable: String,
        /// Variant keys found in the translation.
        found: Vec<String>,
    },
    /// A Boolean variable is referenced outside its selector, which would
    /// render its encoded `true` / `false` string directly.
    BoolReferenceOutsideSelector {
        /// The message id.
        message: String,
        /// The attribute, for an attribute accessor.
        attribute: Option<String>,
        /// The Boolean variable referenced directly.
        variable: String,
    },
    /// A referenced term is not defined in the `.ftl`. Resolving it would
    /// fail to format at runtime.
    UnknownTerm {
        /// The referenced `-term` name.
        term: String,
        /// Where the reference was found: a message id, `message.attribute`,
        /// or another term rendered as `-term`.
        referenced_from: String,
    },
    /// A referenced message (`{ other }`) or attribute (`{ other.title }`) is
    /// not defined. Resolving it would fail to format at runtime.
    UnknownMessageReference {
        /// The reference, rendered as `message` or `message.attribute`.
        reference: String,
        /// Where the reference was found, rendered like
        /// [`ContractViolation::UnknownTerm::referenced_from`].
        referenced_from: String,
    },
    /// A referenced message (`{ other }`) exists but has only attributes, no
    /// value of its own. Resolving it would fail to format at runtime.
    ReferencedMessageHasNoValue {
        /// The referenced message.
        reference: String,
        /// Where the reference was found.
        referenced_from: String,
    },
    /// A referenced term attribute (`{ -brand.case -> … }`) is not defined.
    /// Resolving it would fail to format at runtime.
    UnknownTermAttribute {
        /// The reference, rendered as `-term.attribute`.
        reference: String,
        /// Where the reference was found.
        referenced_from: String,
    },
    /// A term uses a variable after referencing another term. fluent-bundle
    /// then looks the variable up in the accessor's arguments instead of the
    /// term's own, even ones passed explicitly (`{ -term(x: 1) }`); the
    /// accessor doesn't pass it, so formatting fails at runtime. Moving the
    /// variable before the nested term avoids it.
    TermVariableAfterNestedTerm {
        /// The term, rendered as `-term` or `-term.attribute`.
        term: String,
        /// The `$variable` name.
        variable: String,
        /// The accessor's message, rendered as `message` or
        /// `message.attribute`.
        message: String,
    },
    /// A function other than `NUMBER()` is called. The runtime registers only
    /// `NUMBER`, so any other function fails to format.
    UnknownFunction {
        /// The function name, such as `DATETIME`.
        function: String,
        /// Where the call was found.
        referenced_from: String,
    },
    /// Messages or terms reference each other in a loop, which fails to
    /// format at runtime.
    CyclicReference {
        /// The loop, starting and ending with the same entry, such as
        /// `["a", "b", "a"]`. Terms are rendered as `-term`.
        cycle: Vec<String>,
    },
    /// A referenced message uses a variable the accessor doesn't pass. A
    /// referenced message sees the caller's arguments, so it would be unfilled
    /// and fail to format.
    UnknownVariableInReference {
        /// The accessor's message, rendered as `message` or
        /// `message.attribute`.
        message: String,
        /// The referenced message that uses the variable.
        reference: String,
        /// The `$variable` name.
        variable: String,
    },
}

impl fmt::Display for ContractViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingMessage { message } => {
                write!(f, "message '{message}' is missing")
            }
            Self::MissingValue { message } => {
                write!(f, "message '{message}' has no value of its own")
            }
            Self::MissingAttribute { message, attribute } => {
                write!(f, "message '{message}' has no attribute '{attribute}'")
            }
            Self::UnknownVariable {
                message,
                attribute,
                variable,
            } => {
                match attribute {
                    Some(a) => write!(f, "attribute '{a}' of message '{message}'")?,
                    None => write!(f, "message '{message}'")?,
                }
                write!(f, " references unknown variable '${variable}'")
            }
            Self::ElementMismatch {
                message,
                expected,
                found,
            } => write!(
                f,
                "message '{message}' has element markers [{}] but the contract expects [{}]",
                found.join(", "),
                expected.join(", "),
            ),
            Self::BoolSelectorMismatch {
                message,
                attribute,
                variable,
                found,
            } => {
                match attribute {
                    Some(a) => write!(f, "attribute '{a}' of message '{message}'")?,
                    None => write!(f, "message '{message}'")?,
                }
                write!(
                    f,
                    " has Boolean selector ${variable} with keys [{}]; expected [true, false]",
                    found.join(", "),
                )
            }
            Self::BoolReferenceOutsideSelector {
                message,
                attribute,
                variable,
            } => {
                match attribute {
                    Some(a) => write!(f, "attribute '{a}' of message '{message}'")?,
                    None => write!(f, "message '{message}'")?,
                }
                write!(
                    f,
                    " references Boolean variable '${variable}' outside a [true]/[false] selector",
                )
            }
            Self::UnknownTerm {
                term,
                referenced_from,
            } => write!(
                f,
                "'{referenced_from}' references term '-{term}', which is not defined",
            ),
            Self::UnknownMessageReference {
                reference,
                referenced_from,
            } => write!(
                f,
                "'{referenced_from}' references message '{reference}', which is not defined",
            ),
            Self::ReferencedMessageHasNoValue {
                reference,
                referenced_from,
            } => write!(
                f,
                "'{referenced_from}' references message '{reference}', which has only \
                 attributes and no value of its own",
            ),
            Self::UnknownTermAttribute {
                reference,
                referenced_from,
            } => write!(
                f,
                "'{referenced_from}' references term attribute '{reference}', which is not \
                 defined",
            ),
            Self::TermVariableAfterNestedTerm {
                term,
                variable,
                message,
            } => write!(
                f,
                "term '{term}' uses '${variable}' after referencing another term, where \
                 '${variable}' is looked up in the arguments of '{message}', which doesn't \
                 pass it; move '${variable}' before the nested term",
            ),
            Self::UnknownFunction {
                function,
                referenced_from,
            } => write!(
                f,
                "'{referenced_from}' calls function {function}(), which is not available \
                 (only NUMBER() is)",
            ),
            Self::CyclicReference { cycle } => {
                write!(f, "references form a cycle: {}", cycle.join(" -> "))
            }
            Self::UnknownVariableInReference {
                message,
                reference,
                variable,
            } => write!(
                f,
                "'{message}' references message '{reference}', which uses variable \
                 '${variable}' that '{message}' doesn't pass",
            ),
        }
    }
}

/// Validate external `.ftl` bytes against the compiled message contracts.
///
/// This is the check behind the generated `L10nLanguage::new_external`. It
/// verifies that:
///
/// - the bytes are valid UTF-8 and parse as Fluent
///   ([`L10nError::InvalidUtf8`] / [`L10nError::ResourceParse`] otherwise);
/// - every contract message (and attribute) is defined;
/// - no validated pattern references a variable outside its contract
///   arguments;
/// - selectors driven by Boolean arguments keep exactly `[true]` and `[false]`
///   variants;
/// - structured messages keep the exact `(Element)` marker sequence;
/// - every message and term referenced from a validated pattern
///   (transitively) is defined, with no reference cycles;
/// - every function called is `NUMBER()`, the only one the runtime registers;
/// - a referenced message uses only variables the accessor passes (it sees the
///   caller's arguments).
///
/// Contract violations are *collected* — the returned
/// [`L10nError::Validation`] lists every problem, not just the first — so a
/// translator can fix a rejected file in one pass.
pub fn validate_ftl(bytes: &[u8], contracts: &[MessageContract]) -> Result<(), L10nError> {
    // Borrow the bytes as `&str` without copying; only on invalid UTF-8 is a
    // copy made, because `L10nError::InvalidUtf8` carries a `FromUtf8Error`.
    let ftl = std::str::from_utf8(bytes)
        .map_err(|_| L10nError::InvalidUtf8(String::from_utf8(bytes.to_vec()).unwrap_err()))?;
    let resource = match parser::parse(ftl) {
        Ok(resource) => resource,
        Err((_, errors)) => {
            return Err(L10nError::ResourceParse(
                errors.iter().map(|e| format!("{e:?}")).collect(),
            ));
        }
    };

    let (messages, terms) = index_entries(&resource);

    let mut violations = Vec::new();
    // One resolver for all contracts, so a broken entry reached from several
    // messages is reported once.
    let graph = AstGraph { messages, terms };
    let mut resolver = Resolver::new(&graph);

    for contract in contracts {
        let Some(message) = graph.messages.get(contract.message) else {
            violations.push(ContractViolation::MissingMessage {
                message: contract.message.to_string(),
            });
            continue;
        };

        let pattern = if let Some(attribute) = contract.attribute {
            match message.attributes.iter().find(|a| a.id.name == attribute) {
                Some(attr) => &attr.value,
                None => {
                    violations.push(ContractViolation::MissingAttribute {
                        message: contract.message.to_string(),
                        attribute: attribute.to_string(),
                    });
                    continue;
                }
            }
        } else {
            match message.value.as_ref() {
                Some(value) => value,
                None => {
                    violations.push(ContractViolation::MissingValue {
                        message: contract.message.to_string(),
                    });
                    continue;
                }
            }
        };

        let (refs, selectors) = find_refs_and_selectors(pattern);
        let elements: Vec<(&str, RefKind)> = contract
            .elements
            .iter()
            .map(|e| {
                let kind = if e.is_term {
                    RefKind::Term
                } else {
                    RefKind::Variable
                };
                (e.name, kind)
            })
            .collect();
        if let Err(incompatibilities) = check_refs(
            contract.vars,
            contract.bool_vars,
            &elements,
            &refs,
            &selectors,
        ) {
            violations.extend(
                incompatibilities
                    .into_iter()
                    .map(|incompat| match incompat {
                        RefsIncompat::UnknownVariable { variable } => {
                            ContractViolation::UnknownVariable {
                                message: contract.message.to_string(),
                                attribute: contract.attribute.map(str::to_string),
                                variable,
                            }
                        }
                        RefsIncompat::ElementMismatch { expected, found } => {
                            ContractViolation::ElementMismatch {
                                message: contract.message.to_string(),
                                expected: render_elements(&expected),
                                found: render_elements(&found),
                            }
                        }
                        RefsIncompat::BoolSelectorMismatch { variable, found } => {
                            ContractViolation::BoolSelectorMismatch {
                                message: contract.message.to_string(),
                                attribute: contract.attribute.map(str::to_string),
                                variable,
                                found,
                            }
                        }
                        RefsIncompat::BoolReferenceOutsideSelector { variable } => {
                            ContractViolation::BoolReferenceOutsideSelector {
                                message: contract.message.to_string(),
                                attribute: contract.attribute.map(str::to_string),
                                variable,
                            }
                        }
                    }),
            );
        }

        resolver.check(contract.message, contract.attribute, &refs, contract.vars);
    }
    violations.extend(resolver.into_violations());

    if violations.is_empty() {
        Ok(())
    } else {
        Err(L10nError::Validation { violations })
    }
}

/// The messages and terms of a parsed resource, by id. The first definition
/// wins, as in fluent-bundle.
pub(crate) type Messages<'a> = FxHashMap<&'a str, &'a ast::Message<&'a str>>;
/// See [`Messages`].
pub(crate) type Terms<'a> = FxHashMap<&'a str, &'a ast::Term<&'a str>>;

/// Index a parsed resource's messages and terms by id.
pub(crate) fn index_entries<'a>(resource: &'a ast::Resource<&'a str>) -> (Messages<'a>, Terms<'a>) {
    // Pre-sized so the maps never rehash while they fill (entries is an upper
    // bound for both).
    let capacity = resource.body.len();
    let mut messages: Messages = FxHashMap::with_capacity_and_hasher(capacity, Default::default());
    let mut terms: Terms = FxHashMap::with_capacity_and_hasher(capacity, Default::default());
    for entry in &resource.body {
        match entry {
            ast::Entry::Message(m) => {
                messages.entry(m.id.name).or_insert(m);
            }
            ast::Entry::Term(t) => {
                terms.entry(t.id.name).or_insert(t);
            }
            _ => {}
        }
    }
    (messages, terms)
}

/// The pattern of `message`, or of its `attribute`, if defined.
pub(crate) fn find_pattern<'a>(
    messages: &Messages<'a>,
    message: &str,
    attribute: Option<&str>,
) -> Option<&'a ast::Pattern<&'a str>> {
    let m = messages.get(message)?;
    match attribute {
        Some(a) => m
            .attributes
            .iter()
            .find(|x| x.id.name == a)
            .map(|x| &x.value),
        None => m.value.as_ref(),
    }
}

/// What the [`Resolver`] needs to know about a translation: the references of
/// each message pattern and term, and whether they exist.
pub(crate) trait RefGraph {
    /// The references of `message`'s value, or of its `attribute`; `None`
    /// when that pattern isn't defined.
    fn message_refs(&self, message: &str, attribute: Option<&str>) -> Option<Cow<'_, [Ref]>>;
    /// Whether `message` is defined at all (with or without a value).
    fn message_exists(&self, message: &str) -> bool;
    /// The references of the term `name` (without `-`), or of its `attribute`;
    /// `None` when that pattern isn't defined.
    fn term_refs(&self, name: &str, attribute: Option<&str>) -> Option<Cow<'_, [Ref]>>;
}

/// A [`RefGraph`] over a parsed resource, as [`validate_ftl`] has. References
/// are collected when a pattern is walked.
pub(crate) struct AstGraph<'a> {
    messages: Messages<'a>,
    terms: Terms<'a>,
}

impl RefGraph for AstGraph<'_> {
    fn message_refs(&self, message: &str, attribute: Option<&str>) -> Option<Cow<'_, [Ref]>> {
        let pattern = find_pattern(&self.messages, message, attribute)?;
        Some(Cow::Owned(find_refs_and_selectors(pattern).0))
    }

    fn message_exists(&self, message: &str) -> bool {
        self.messages.contains_key(message)
    }

    fn term_refs(&self, name: &str, attribute: Option<&str>) -> Option<Cow<'_, [Ref]>> {
        let term = self.terms.get(name)?;
        let pattern = match attribute {
            Some(a) => &term.attributes.iter().find(|x| x.id.name == a)?.value,
            None => &term.value,
        };
        Some(Cow::Owned(find_refs_and_selectors(pattern).0))
    }
}

/// Checks that everything a generated accessor's pattern references resolves:
/// messages, attributes and terms exist (transitively, without cycles), every
/// function is one the runtime registers, and variables that end up looked up
/// in the accessor's arguments are ones it passes.
///
/// Used by [`validate_ftl`] and by the build, for every locale, so both hold
/// translations to the same rules. Only entries reachable from the checked
/// patterns are walked, so an unused, broken message or term doesn't count.
pub(crate) struct Resolver<'g, G: RefGraph> {
    graph: &'g G,
    violations: Vec<ContractViolation>,
    reported: FxHashSet<ContractViolation>,
}

/// Where a walk is: the entries on the current reference path (for cycle
/// detection), the entries already walked for this accessor, and the
/// accessor's own name and arguments.
struct Walk<'s> {
    stack: Vec<String>,
    visited: FxHashSet<String>,
    accessor: String,
    allowed: &'s [&'s str],
}

impl<'g, G: RefGraph> Resolver<'g, G> {
    pub(crate) fn new(graph: &'g G) -> Self {
        Self {
            graph,
            violations: Vec::new(),
            reported: FxHashSet::default(),
        }
    }

    /// Check the pattern of `message` (or its `attribute`), whose references
    /// are `refs` and whose accessor passes the variables in `allowed`, its
    /// arguments. `(Element)` variables are not arguments: they're never
    /// passed, so a referenced message can't use them.
    ///
    /// The pattern's own variables are not checked here; that's [`check_refs`].
    pub(crate) fn check(
        &mut self,
        message: &str,
        attribute: Option<&str>,
        refs: &[Ref],
        allowed: &[&str],
    ) {
        // Nothing to resolve: the common case, kept cheap.
        if refs.iter().all(|r| r.kind == RefKind::Variable) {
            return;
        }
        let accessor = render_reference(message, attribute);
        let mut walk = Walk {
            stack: vec![accessor.clone()],
            visited: FxHashSet::default(),
            accessor: accessor.clone(),
            allowed,
        };
        self.walk(refs, &accessor, Scope::Accessor, &mut walk);
    }

    pub(crate) fn into_violations(self) -> Vec<ContractViolation> {
        self.violations
    }

    fn report(&mut self, violation: ContractViolation) {
        if self.reported.insert(violation.clone()) {
            self.violations.push(violation);
        }
    }

    fn walk(&mut self, refs: &[Ref], origin: &str, scope: Scope, walk: &mut Walk<'_>) {
        // Inside a term, fluent-bundle forgets the term's own arguments once a
        // nested term has been resolved, and looks later variables up in the
        // accessor's arguments. Refs are in document order, so this tracks
        // "after a nested term" (conservatively across select variants).
        let mut after_nested_term = false;
        for r in refs {
            match r.kind {
                RefKind::Function => {
                    if !KNOWN_FUNCTIONS.contains(&r.name.as_str()) {
                        self.report(ContractViolation::UnknownFunction {
                            function: r.name.clone(),
                            referenced_from: origin.to_string(),
                        });
                    }
                }
                // The accessor's own variables are `check_refs`' job.
                RefKind::Variable => {
                    let passed = walk.allowed.contains(&r.name.as_str());
                    match scope {
                        Scope::Accessor => {}
                        Scope::ReferencedMessage if !passed => {
                            self.report(ContractViolation::UnknownVariableInReference {
                                message: walk.accessor.clone(),
                                reference: origin.to_string(),
                                variable: r.name.clone(),
                            });
                        }
                        // Before a nested term, a term's variable is its own
                        // argument; if missing it renders as `{$x}`, no panic.
                        Scope::Term if after_nested_term && !passed => {
                            self.report(ContractViolation::TermVariableAfterNestedTerm {
                                term: origin.to_string(),
                                variable: r.name.clone(),
                                message: walk.accessor.clone(),
                            });
                        }
                        Scope::ReferencedMessage | Scope::Term => {}
                    }
                }
                RefKind::Term | RefKind::TermAttribute => {
                    self.term(r, origin, walk);
                    after_nested_term = true;
                }
                RefKind::Message => self.message(r, origin, scope, walk),
            }
        }
    }

    fn term(&mut self, r: &Ref, origin: &str, walk: &mut Walk<'_>) {
        let (name, attribute) = split_reference(&r.name);
        let graph = self.graph;
        let Some(refs) = graph.term_refs(name, attribute) else {
            let violation = if attribute.is_some() && graph.term_refs(name, None).is_some() {
                ContractViolation::UnknownTermAttribute {
                    reference: format!("-{}", r.name),
                    referenced_from: origin.to_string(),
                }
            } else {
                ContractViolation::UnknownTerm {
                    term: name.to_string(),
                    referenced_from: origin.to_string(),
                }
            };
            self.report(violation);
            return;
        };
        let key = format!("-{}", r.name);
        if self.enter_cycle(&key, walk) {
            return;
        }
        // Per accessor: whether a term is fine depends on the accessor's
        // arguments (see `TermVariableAfterNestedTerm`).
        if walk.visited.insert(key.clone()) {
            walk.stack.push(key.clone());
            self.walk(&refs, &key, Scope::Term, walk);
            walk.stack.pop();
        }
    }

    fn message(&mut self, r: &Ref, origin: &str, scope: Scope, walk: &mut Walk<'_>) {
        let (id, attribute) = split_reference(&r.name);
        let graph = self.graph;
        let Some(refs) = graph.message_refs(id, attribute) else {
            let violation = if attribute.is_none() && graph.message_exists(id) {
                ContractViolation::ReferencedMessageHasNoValue {
                    reference: r.name.clone(),
                    referenced_from: origin.to_string(),
                }
            } else {
                ContractViolation::UnknownMessageReference {
                    reference: r.name.clone(),
                    referenced_from: origin.to_string(),
                }
            };
            self.report(violation);
            return;
        };
        if self.enter_cycle(&r.name, walk) {
            return;
        }
        // Inside a term, a message reference is resolved in the term's scope,
        // not the accessor's, so its variables aren't checked.
        let inner = match scope {
            Scope::Term => Scope::Term,
            _ => Scope::ReferencedMessage,
        };
        if walk.visited.insert(format!("{inner:?}:{}", r.name)) {
            walk.stack.push(r.name.clone());
            self.walk(&refs, &r.name, inner, walk);
            walk.stack.pop();
        }
    }

    /// Report and return `true` if entering `key` closes a reference cycle.
    fn enter_cycle(&mut self, key: &str, walk: &Walk<'_>) -> bool {
        let Some(start) = walk.stack.iter().position(|k| k == key) else {
            return false;
        };
        let mut cycle = walk.stack[start..].to_vec();
        cycle.push(key.to_string());
        self.report(ContractViolation::CyclicReference { cycle });
        true
    }
}

/// Whose arguments a pattern sees while walking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    /// The accessor's own pattern.
    Accessor,
    /// A message referenced (directly or not) from the accessor's pattern:
    /// it sees the accessor's arguments.
    ReferencedMessage,
    /// A term, or anything referenced from one: its own arguments, until a
    /// nested term is resolved.
    Term,
}

/// Split `name` or `name.attribute`.
fn split_reference(reference: &str) -> (&str, Option<&str>) {
    match reference.split_once('.') {
        Some((name, attribute)) => (name, Some(attribute)),
        None => (reference, None),
    }
}

/// Render a reference as `message` or `message.attribute`.
fn render_reference(message: &str, attribute: Option<&str>) -> String {
    match attribute {
        Some(a) => format!("{message}.{a}"),
        None => message.to_string(),
    }
}

/// Render an element marker sequence as `$variable` / `-term` names.
fn render_elements(elements: &[(String, RefKind)]) -> Vec<String> {
    elements
        .iter()
        .map(|(name, kind)| match kind {
            RefKind::Variable => format!("${name}"),
            RefKind::Term => format!("-{name}"),
            // Elements are only ever variables or terms.
            RefKind::TermAttribute | RefKind::Message | RefKind::Function => name.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTRACTS: &[MessageContract] = &[
        MessageContract {
            message: "hello",
            attribute: None,
            vars: &["name"],
            bool_vars: &[],
            elements: &[],
        },
        MessageContract {
            message: "login",
            attribute: Some("placeholder"),
            vars: &[],
            bool_vars: &[],
            elements: &[],
        },
        MessageContract {
            message: "notice",
            attribute: None,
            vars: &["count"],
            bool_vars: &[],
            elements: &[
                ElementContract {
                    name: "icon",
                    is_term: false,
                },
                ElementContract {
                    name: "privacy-link",
                    is_term: true,
                },
            ],
        },
    ];

    const VALID: &str = r#"
hello = Hej { $name }
login = Logga in
    .placeholder = Ange din e-post
notice = { $icon } Du har { $count } olästa, se { -privacy-link }.
-privacy-link = vår integritetspolicy
"#;

    fn violations(ftl: &str) -> Vec<ContractViolation> {
        match validate_ftl(ftl.as_bytes(), CONTRACTS) {
            Err(L10nError::Validation { violations }) => violations,
            other => panic!("expected Validation error, got {other:?}"),
        }
    }

    #[test]
    fn a_complete_translation_passes() {
        validate_ftl(VALID.as_bytes(), CONTRACTS).unwrap();
    }

    #[test]
    fn fewer_variables_than_the_contract_is_allowed() {
        // A locale that does not need an argument is fine — build-time
        // locales are held to the same rule.
        let ftl = VALID.replace("Hej { $name }", "Hej!");
        validate_ftl(ftl.as_bytes(), CONTRACTS).unwrap();
    }

    #[test]
    fn a_missing_message_is_a_violation() {
        let ftl = VALID.replace("hello", "helo");
        assert_eq!(
            violations(&ftl),
            vec![ContractViolation::MissingMessage {
                message: "hello".to_string()
            }]
        );
    }

    #[test]
    fn a_missing_attribute_is_a_violation() {
        let ftl = VALID.replace(".placeholder", ".placeholdr");
        assert_eq!(
            violations(&ftl),
            vec![ContractViolation::MissingAttribute {
                message: "login".to_string(),
                attribute: "placeholder".to_string(),
            }]
        );
    }

    #[test]
    fn a_message_without_a_value_is_a_violation() {
        let ftl = VALID.replace(
            "hello = Hej { $name }",
            "hello =\n    .other = Hej { $name }",
        );
        assert_eq!(
            violations(&ftl),
            vec![ContractViolation::MissingValue {
                message: "hello".to_string()
            }]
        );
    }

    #[test]
    fn an_unknown_variable_is_a_violation() {
        // `$nom` is not a contract argument: the accessor would leave it
        // unfilled. This is the typo that used to panic at accessor time.
        let ftl = VALID.replace("{ $name }", "{ $nom }");
        assert_eq!(
            violations(&ftl),
            vec![ContractViolation::UnknownVariable {
                message: "hello".to_string(),
                attribute: None,
                variable: "nom".to_string(),
            }]
        );
    }

    #[test]
    fn an_unknown_variable_in_a_selector_is_a_violation() {
        let ftl = VALID.replace(
            "hello = Hej { $name }",
            "hello = { $other ->\n    [one] En\n   *[other] Hej\n}",
        );
        assert_eq!(
            violations(&ftl),
            vec![ContractViolation::UnknownVariable {
                message: "hello".to_string(),
                attribute: None,
                variable: "other".to_string(),
            }]
        );
    }

    #[test]
    fn boolean_selector_keys_are_validated() {
        const BOOL_CONTRACT: &[MessageContract] = &[MessageContract {
            message: "feature",
            attribute: None,
            vars: &["enabled"],
            bool_vars: &["enabled"],
            elements: &[],
        }];
        let valid = "feature = { $enabled ->\n    [true] On\n   *[false] Off\n}\n";
        validate_ftl(valid.as_bytes(), BOOL_CONTRACT).unwrap();

        let invalid = "feature = { $enabled ->\n    [yes] On\n   *[no] Off\n}\n";
        match validate_ftl(invalid.as_bytes(), BOOL_CONTRACT) {
            Err(L10nError::Validation { violations }) => assert_eq!(
                violations,
                vec![ContractViolation::BoolSelectorMismatch {
                    message: "feature".to_string(),
                    attribute: None,
                    variable: "enabled".to_string(),
                    found: vec!["yes".to_string(), "no".to_string()],
                }]
            ),
            other => panic!("expected Validation error, got {other:?}"),
        }
    }

    #[test]
    fn boolean_variable_must_not_be_interpolated_directly() {
        const BOOL_CONTRACT: &[MessageContract] = &[MessageContract {
            message: "feature",
            attribute: None,
            vars: &["enabled"],
            bool_vars: &["enabled"],
            elements: &[],
        }];
        let invalid = "feature = Feature: { $enabled }\n";

        match validate_ftl(invalid.as_bytes(), BOOL_CONTRACT) {
            Err(L10nError::Validation { violations }) => assert_eq!(
                violations,
                vec![ContractViolation::BoolReferenceOutsideSelector {
                    message: "feature".to_string(),
                    attribute: None,
                    variable: "enabled".to_string(),
                }]
            ),
            other => panic!("expected Validation error, got {other:?}"),
        }
    }

    #[test]
    fn independent_reference_violations_are_collected() {
        const BOOL_CONTRACT: &[MessageContract] = &[MessageContract {
            message: "feature",
            attribute: None,
            vars: &["enabled"],
            bool_vars: &["enabled"],
            elements: &[],
        }];
        let invalid = "feature = { $enabled ->\n    [yes] { $extra }\n   *[no] Off\n}\n";

        match validate_ftl(invalid.as_bytes(), BOOL_CONTRACT) {
            Err(L10nError::Validation { violations }) => assert_eq!(
                violations,
                vec![
                    ContractViolation::UnknownVariable {
                        message: "feature".to_string(),
                        attribute: None,
                        variable: "extra".to_string(),
                    },
                    ContractViolation::BoolSelectorMismatch {
                        message: "feature".to_string(),
                        attribute: None,
                        variable: "enabled".to_string(),
                        found: vec!["yes".to_string(), "no".to_string()],
                    },
                ]
            ),
            other => panic!("expected Validation error, got {other:?}"),
        }
    }

    #[test]
    fn a_reordered_element_sequence_is_a_violation() {
        let ftl = VALID.replace(
            "{ $icon } Du har { $count } olästa, se { -privacy-link }.",
            "Se { -privacy-link } { $icon } för { $count } olästa.",
        );
        assert_eq!(
            violations(&ftl),
            vec![ContractViolation::ElementMismatch {
                message: "notice".to_string(),
                expected: vec!["$icon".to_string(), "-privacy-link".to_string()],
                found: vec!["-privacy-link".to_string(), "$icon".to_string()],
            }]
        );
    }

    #[test]
    fn a_dropped_element_is_a_violation() {
        let ftl = VALID.replace("{ $icon } ", "");
        assert_eq!(
            violations(&ftl),
            vec![ContractViolation::ElementMismatch {
                message: "notice".to_string(),
                expected: vec!["$icon".to_string(), "-privacy-link".to_string()],
                found: vec!["-privacy-link".to_string()],
            }]
        );
    }

    #[test]
    fn an_undefined_term_is_a_violation() {
        let ftl = VALID.replace("-privacy-link = vår integritetspolicy", "");
        assert_eq!(
            violations(&ftl),
            vec![ContractViolation::UnknownTerm {
                term: "privacy-link".to_string(),
                referenced_from: "notice".to_string(),
            }]
        );
    }

    #[test]
    fn an_undefined_term_behind_another_term_is_a_violation() {
        let ftl = VALID.replace(
            "-privacy-link = vår integritetspolicy",
            "-privacy-link = vår { -policy }",
        );
        assert_eq!(
            violations(&ftl),
            vec![ContractViolation::UnknownTerm {
                term: "policy".to_string(),
                referenced_from: "-privacy-link".to_string(),
            }]
        );
    }

    #[test]
    fn all_violations_are_collected() {
        let ftl = "notice = Bara { $typo } kvar\n";
        let found = violations(ftl);
        // Violations are collected both across and within messages: hello and
        // login are missing; notice dropped its markers and added `$typo`.
        assert_eq!(found.len(), 4, "got: {found:?}");
        assert!(found.iter().any(
            |v| matches!(v, ContractViolation::MissingMessage { message } if message == "hello")
        ));
        assert!(found.iter().any(
            |v| matches!(v, ContractViolation::ElementMismatch { message, .. } if message == "notice")
        ));
        assert!(found.iter().any(
            |v| matches!(v, ContractViolation::UnknownVariable { variable, .. } if variable == "typo")
        ));
    }

    #[test]
    fn a_parse_error_is_a_resource_parse_error() {
        let err = validate_ftl(b"hello = { $unclosed\n", CONTRACTS).unwrap_err();
        assert!(matches!(err, L10nError::ResourceParse(_)), "got: {err:?}");
    }

    #[test]
    fn invalid_utf8_is_an_utf8_error() {
        let err = validate_ftl(&[0xff, 0xfe], CONTRACTS).unwrap_err();
        assert!(matches!(err, L10nError::InvalidUtf8(_)), "got: {err:?}");
    }

    /// `VALID`, with `hello`'s value replaced and extra entries appended.
    fn with_hello(value: &str, extra: &str) -> String {
        VALID.replace("hello = Hej { $name }", &format!("hello = {value}")) + extra
    }

    #[test]
    fn a_reference_to_a_missing_message_is_rejected() {
        let ftl = with_hello("{ $name }, se { inbox-link }", "");
        assert_eq!(
            violations(&ftl),
            [ContractViolation::UnknownMessageReference {
                reference: "inbox-link".into(),
                referenced_from: "hello".into(),
            }]
        );
    }

    #[test]
    fn a_reference_to_a_missing_attribute_or_value_is_rejected() {
        let ftl = with_hello(
            "{ $name } { other.title } { attrs-only }",
            "other = Other\nattrs-only =\n    .a = A\n",
        );
        assert_eq!(
            violations(&ftl),
            [
                ContractViolation::UnknownMessageReference {
                    reference: "other.title".into(),
                    referenced_from: "hello".into(),
                },
                ContractViolation::ReferencedMessageHasNoValue {
                    reference: "attrs-only".into(),
                    referenced_from: "hello".into(),
                },
            ]
        );
    }

    #[test]
    fn a_referenced_message_cannot_use_element_variables() {
        // `$icon` is an `(Element)` slot of `notice`, never passed as an
        // argument, so `help` can't resolve it.
        let ftl = VALID.replace("se { -privacy-link }", "se { -privacy-link } { help }")
            + "help = Klicka { $icon }\n";
        assert_eq!(
            violations(&ftl),
            [ContractViolation::UnknownVariableInReference {
                message: "notice".into(),
                reference: "help".into(),
                variable: "icon".into(),
            }]
        );
    }

    #[test]
    fn term_attributes_must_exist_and_resolve() {
        let selector = |attr: &str| {
            with_hello(
                &format!("{{ $name }} {{ -brand.{attr} ->\n   *[x] y\n}}"),
                "-brand = B\n    .gen = { DATETIME(1) }\n",
            )
        };
        assert_eq!(
            violations(&selector("case")),
            [ContractViolation::UnknownTermAttribute {
                reference: "-brand.case".into(),
                referenced_from: "hello".into(),
            }]
        );
        assert_eq!(
            violations(&selector("gen")),
            [ContractViolation::UnknownFunction {
                function: "DATETIME".into(),
                referenced_from: "-brand.gen".into(),
            }]
        );
    }

    #[test]
    fn a_term_variable_after_a_nested_term_must_be_an_accessor_argument() {
        // fluent-bundle looks `$x` up in the accessor's arguments once `-inner`
        // has been resolved, even though `-outer` is called with `x`.
        let ftl = with_hello(
            "{ $name } { -outer(x: 1) }",
            "-inner = hi\n-outer = { -inner } { $x }\n",
        );
        assert_eq!(
            violations(&ftl),
            [ContractViolation::TermVariableAfterNestedTerm {
                term: "-outer".into(),
                variable: "x".into(),
                message: "hello".into(),
            }]
        );
        // Before the nested term, or when the accessor passes it, it's fine.
        let before = with_hello(
            "{ $name } { -outer(x: 1) }",
            "-inner = hi\n-outer = { $x } { -inner }\n",
        );
        validate_ftl(before.as_bytes(), CONTRACTS).unwrap();
        let passed = with_hello(
            "{ $name } { -outer }",
            "-inner = hi\n-outer = { -inner } { $name }\n",
        );
        validate_ftl(passed.as_bytes(), CONTRACTS).unwrap();
    }

    #[test]
    fn a_missing_message_is_found_through_a_chain_of_references() {
        let ftl = with_hello(
            "{ $name } { a }",
            "a = { b }\nb = { -t }\n-t = { missing }\n",
        );
        assert_eq!(
            violations(&ftl),
            [ContractViolation::UnknownMessageReference {
                reference: "missing".into(),
                referenced_from: "-t".into(),
            }]
        );
    }

    #[test]
    fn a_function_other_than_number_is_rejected() {
        let ftl = with_hello("{ $name } { DATETIME($name) } { NUMBER(3) }", "");
        assert_eq!(
            violations(&ftl),
            [ContractViolation::UnknownFunction {
                function: "DATETIME".into(),
                referenced_from: "hello".into(),
            }]
        );
    }

    #[test]
    fn reference_cycles_are_rejected() {
        let ftl = with_hello("{ $name } { a }", "a = { b }\nb = { a }\n");
        assert_eq!(
            violations(&ftl),
            [ContractViolation::CyclicReference {
                cycle: vec!["a".into(), "b".into(), "a".into()],
            }]
        );

        let ftl = with_hello("{ $name } { -a }", "-a = { -b }\n-b = { -a }\n");
        assert_eq!(
            violations(&ftl),
            [ContractViolation::CyclicReference {
                cycle: vec!["-a".into(), "-b".into(), "-a".into()],
            }]
        );
    }

    #[test]
    fn a_referenced_message_may_only_use_the_accessors_variables() {
        // `greeting` sees `hello`'s arguments: `$name` is passed, `$title` isn't.
        let ftl = with_hello("{ greeting }", "greeting = Hi { $title } { $name }\n");
        assert_eq!(
            violations(&ftl),
            [ContractViolation::UnknownVariableInReference {
                message: "hello".into(),
                reference: "greeting".into(),
                variable: "title".into(),
            }]
        );
        // A term's variables are its own arguments, not the accessor's.
        let ftl = with_hello("{ $name } { -t(x: 1) }", "-t = { $x }\n");
        validate_ftl(ftl.as_bytes(), CONTRACTS).unwrap();
    }

    #[test]
    fn working_references_and_unused_broken_entries_pass() {
        let ftl = with_hello(
            "{ greeting } { other.title } { NUMBER(2) }",
            "greeting = Hi { $name }\nother = O\n    .title = T\n\
             unused = { missing } { DATETIME(1) }\n",
        );
        validate_ftl(ftl.as_bytes(), CONTRACTS).unwrap();
    }

    #[test]
    fn violations_render_readably() {
        let ftl = VALID.replace("{ $name }", "{ $nom }");
        let err = validate_ftl(ftl.as_bytes(), CONTRACTS).unwrap_err();
        let text = err.to_string();
        assert!(
            text.contains("message 'hello' references unknown variable '$nom'"),
            "got: {text}"
        );
    }
}
