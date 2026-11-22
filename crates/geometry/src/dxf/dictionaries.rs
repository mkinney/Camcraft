//! Provenance: dictionary entries, handles and ownership in the Autodesk DXF Reference.
use super::pairs::{Pairs, Record, handle, integer};
use super::types::{DxfError, error};
use std::collections::{BTreeMap, BTreeSet};

fn invalid(message: &str) -> DxfError {
    error("DICTIONARY", message)
}

pub(super) fn read(input: &mut Pairs<'_>, limit: usize) -> Result<(), DxfError> {
    let mut owners = BTreeMap::new();
    let mut entries = BTreeMap::new();
    loop {
        let record = input
            .record()?
            .ok_or_else(|| error("OBJECTS", "missing ENDSEC"))?;
        match record.kind {
            "ENDSEC" => {
                record.empty()?;
                return check_links(owners, entries);
            }
            "DICTIONARY" => {
                if owners.len() >= limit {
                    return Err(invalid("dictionary count exceeds limit"));
                }
                let id = handle(record.required(5)?, record.kind)?;
                let owner = read_fields(record, id, &mut entries, limit)?;
                if id == 0 || owners.insert(id, owner).is_some() {
                    return Err(invalid("zero or duplicate dictionary handle"));
                }
            }
            _ => {
                return Err(error(
                    record.kind,
                    "unsupported record in OBJECTS; expected DICTIONARY or ENDSEC",
                ));
            }
        }
    }
}

fn read_fields(
    record: Record<'_>,
    id: u64,
    entries: &mut BTreeMap<u64, u64>,
    limit: usize,
) -> Result<Option<u64>, DxfError> {
    let mut owner = None;
    let mut singleton_fields = BTreeSet::new();
    let mut names = BTreeSet::new();
    let mut needs_target = false;
    let mut fields = record.pairs();
    while let Some(pair) = fields.next()? {
        if needs_target && !matches!(pair.code, 350 | 360) {
            return Err(invalid("missing dictionary entry target"));
        }
        if !matches!(pair.code, 3 | 350 | 360) && !singleton_fields.insert(pair.code) {
            return Err(invalid("duplicate dictionary field"));
        }
        match pair.code {
            5 => {}
            100 if pair.value == "AcDbDictionary" => {}
            330 => owner = Some(handle(pair.value, record.kind)?),
            280 if matches!(integer(pair.value, record.kind)?, 0 | 1) => {}
            281 if (0..=5).contains(&integer(pair.value, record.kind)?) => {}
            3 => {
                if entries.len() >= limit {
                    return Err(invalid("dictionary entry count exceeds limit"));
                }
                if pair.value.trim().is_empty() || !names.insert(pair.value.to_ascii_uppercase()) {
                    return Err(invalid("empty or duplicate entry name"));
                }
                needs_target = true;
            }
            350 | 360 if needs_target => {
                let target = handle(pair.value, record.kind)?;
                if entries.insert(target, id).is_some() {
                    return Err(invalid("dictionary entry target has multiple owners"));
                }
                needs_target = false;
            }
            _ => {
                return Err(invalid(&format!(
                    "unsupported dictionary group {} or value",
                    pair.code
                )));
            }
        }
    }
    if needs_target {
        return Err(invalid("missing dictionary entry target"));
    }
    Ok(owner)
}

fn check_links(
    mut owners: BTreeMap<u64, Option<u64>>,
    entries: BTreeMap<u64, u64>,
) -> Result<(), DxfError> {
    // A named entry also identifies a parent when the child omits its owner field.
    for (child, parent) in entries {
        let owner = owners
            .get_mut(&child)
            .ok_or_else(|| invalid("unresolved entry target"))?;
        if owner.is_some_and(|declared| declared != parent) {
            return Err(invalid("entry target names a different owner"));
        }
        *owner = Some(parent);
    }

    // A chain terminates at a root or at a chain validated in a previous walk.
    // Mark each visited dictionary once to bound the work without recursion.
    let mut visited = BTreeMap::new();
    for &start in owners.keys() {
        let mut node = start;
        while node != 0 {
            if let Some(previous_walk) = visited.get(&node) {
                if *previous_walk == start {
                    return Err(invalid("cyclic dictionary ownership"));
                }
                break;
            }
            visited.insert(node, start);
            node = owners
                .get(&node)
                .ok_or_else(|| invalid("unresolved dictionary owner"))?
                .unwrap_or(0);
        }
    }
    Ok(())
}
