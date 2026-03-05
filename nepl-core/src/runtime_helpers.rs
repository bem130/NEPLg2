#![allow(dead_code)]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;

pub const ALLOC_CANDIDATES: &[&str] = &["alloc", "alloc_raw"];
pub const DEALLOC_CANDIDATES: &[&str] = &["dealloc", "dealloc_raw"];
pub const REALLOC_CANDIDATES: &[&str] = &["realloc", "realloc_raw"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeHelperKind {
    Alloc,
    Dealloc,
    Realloc,
}

pub fn helper_candidates(kind: RuntimeHelperKind) -> &'static [&'static str] {
    match kind {
        RuntimeHelperKind::Alloc => ALLOC_CANDIDATES,
        RuntimeHelperKind::Dealloc => DEALLOC_CANDIDATES,
        RuntimeHelperKind::Realloc => REALLOC_CANDIDATES,
    }
}

pub fn helper_base_name(name: &str) -> &str {
    let tail = if let Some(pos) = name.rfind("::") {
        &name[pos + 2..]
    } else {
        name
    };
    if let Some(pos) = tail.find("__") {
        &tail[..pos]
    } else {
        tail
    }
}

pub fn find_runtime_helper_key<'a, T>(
    map: &'a BTreeMap<String, T>,
    kind: RuntimeHelperKind,
) -> Option<&'a str> {
    for base in helper_candidates(kind) {
        if let Some((name, _)) = map.get_key_value(*base) {
            return Some(name.as_str());
        }
        for (name, _) in map {
            if helper_base_name(name.as_str()) == *base {
                return Some(name.as_str());
            }
        }
    }
    None
}

pub fn find_runtime_helper_index(
    name_map: &BTreeMap<String, u32>,
    kind: RuntimeHelperKind,
    current_func: Option<&str>,
) -> Option<u32> {
    let skip_idx = current_func.and_then(|n| name_map.get(n)).copied();
    for base in helper_candidates(kind) {
        if let Some(idx) = name_map.get(*base) {
            if Some(*idx) != skip_idx {
                return Some(*idx);
            }
        }
        for (name, idx) in name_map {
            if Some(*idx) == skip_idx {
                continue;
            }
            if helper_base_name(name.as_str()) == *base {
                return Some(*idx);
            }
        }
    }
    None
}
