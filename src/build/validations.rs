use std::collections::{HashMap, HashSet};

use crate::{
    build::LangBundle,
    build::typed::{Id, Variable},
};

#[derive(Debug)]
pub struct Analyzed {
    pub common: HashSet<Id>,
    pub missing_messages: Vec<String>,
    pub signature_mismatches: Vec<String>,
}
impl Analyzed {
    pub fn from(langs: &[LangBundle]) -> Self {
        let common_ids = common_message_ids(langs);
        let missing_messages = missing_message_ids(&common_ids, langs);
        let (signature_mismatches, ids) = signature_mismatches(&common_ids, langs);
        let common: HashSet<Id> = common_ids.difference(&ids).map(|id| id.clone()).collect();
        Self {
            common,
            missing_messages,
            signature_mismatches,
        }
    }
}

fn signature_mismatches(
    common_ids: &HashSet<Id>,
    langs: &[LangBundle],
) -> (Vec<String>, HashSet<Id>) {
    let mut messages = vec![];
    let mut mismatched_ids = HashSet::new();

    for id in common_ids {
        let signatures = signatures_for_id(id, langs);
        if signatures.len() > 1 {
            mismatched_ids.insert(id.clone());
            let sig_vals = signatures
                .values()
                .map(|v| format!("[{}]", v.join(", ")))
                .collect::<Vec<_>>()
                .join(" != ");

            messages.push(format!(
                "Different signatures for message {id} in languages: {sig_vals}",
            ));
        }
    }
    (messages, mismatched_ids)
}

fn signatures_for_id<'a>(id: &Id, langs: &'a [LangBundle]) -> HashMap<&'a [Variable], Vec<String>> {
    let mut signatures: HashMap<&[Variable], Vec<String>> = HashMap::new();
    for lang in langs {
        for msg in &lang.messages {
            if &msg.id == id {
                signatures
                    .entry(&msg.variables)
                    .or_default()
                    .push(msg.trait_signature());
            }
        }
    }
    signatures
}

fn common_message_ids(langs: &[LangBundle]) -> HashSet<Id> {
    let mut lang_signatures = vec![];

    for lang in langs {
        lang_signatures.push(
            lang.messages
                .iter()
                .map(|msg| msg.id.clone())
                .collect::<HashSet<Id>>(),
        );
    }
    let mut iter = lang_signatures.iter();
    let mut common = iter.next().unwrap().clone();

    for other in iter {
        common = common.intersection(other).cloned().collect();
    }
    common
}

fn missing_message_ids(common_ids: &HashSet<Id>, langs: &[LangBundle]) -> Vec<String> {
    let mut not_present: HashMap<Id, Vec<String>> = HashMap::new();

    for lang in langs {
        for msg in &lang.messages {
            if !common_ids.contains(&msg.id) {
                not_present
                    .entry(msg.id.clone())
                    .or_default()
                    .push(lang.language_id.clone());
            }
        }
    }
    not_present
        .into_iter()
        .map(|(id, v)| format!("Missing {id} for languages: {}", v.join(", ")))
        .collect()
}
