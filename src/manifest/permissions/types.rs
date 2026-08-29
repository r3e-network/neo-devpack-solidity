use super::*;

pub(crate) const MAX_ABSTRACT_OPTIONS: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AbstractAtom {
    Literal(ir::LiteralValue),
    ExecutingScriptHash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AbstractValue {
    Possible(Vec<AbstractAtom>),
    Unknown,
}

impl AbstractValue {
    pub(crate) fn literal(lit: ir::LiteralValue) -> Self {
        Self::Possible(vec![AbstractAtom::Literal(lit)])
    }

    pub(crate) fn executing_script_hash() -> Self {
        Self::Possible(vec![AbstractAtom::ExecutingScriptHash])
    }

    pub(crate) fn atoms(&self) -> Option<&[AbstractAtom]> {
        match self {
            AbstractValue::Possible(values) => Some(values.as_slice()),
            AbstractValue::Unknown => None,
        }
    }

    pub(crate) fn merge(&self, other: &Self) -> Self {
        match (self, other) {
            (AbstractValue::Unknown, _) | (_, AbstractValue::Unknown) => AbstractValue::Unknown,
            (AbstractValue::Possible(left), AbstractValue::Possible(right)) => {
                let mut merged = left.clone();
                for atom in right {
                    if !merged.contains(atom) {
                        merged.push(atom.clone());
                    }
                    if merged.len() > MAX_ABSTRACT_OPTIONS {
                        return AbstractValue::Unknown;
                    }
                }
                AbstractValue::Possible(merged)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AbstractState {
    pub(crate) stack: Vec<AbstractValue>,
    pub(crate) locals: Vec<AbstractValue>,
}

impl AbstractState {
    pub(crate) fn new(local_count: u16) -> Self {
        Self {
            stack: Vec::new(),
            locals: vec![AbstractValue::Unknown; local_count as usize],
        }
    }

    pub(crate) fn merge_from(&mut self, other: &Self) -> bool {
        let mut changed = false;

        if self.stack.len() != other.stack.len() {
            let max_len = self.stack.len().max(other.stack.len());
            let mut merged = vec![AbstractValue::Unknown; max_len];
            for (idx, value) in self.stack.iter().enumerate() {
                merged[idx] = value.clone();
            }
            for (idx, value) in other.stack.iter().enumerate() {
                merged[idx] = merged[idx].merge(value);
            }
            if self.stack != merged {
                self.stack = merged;
                changed = true;
            }
        } else {
            for (slot, incoming) in self.stack.iter_mut().zip(other.stack.iter()) {
                let merged = slot.merge(incoming);
                if *slot != merged {
                    *slot = merged;
                    changed = true;
                }
            }
        }

        if self.locals.len() != other.locals.len() {
            let max_len = self.locals.len().max(other.locals.len());
            self.locals.resize(max_len, AbstractValue::Unknown);
        }

        for (slot, incoming) in self.locals.iter_mut().zip(other.locals.iter()) {
            let merged = slot.merge(incoming);
            if *slot != merged {
                *slot = merged;
                changed = true;
            }
        }

        changed
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PermissionMethods {
    All,
    Some(std::collections::BTreeSet<String>),
}

impl PermissionMethods {
    pub(crate) fn merge_in(&mut self, other: PermissionMethods) {
        match (self, other) {
            (PermissionMethods::All, _) => {}
            (this, PermissionMethods::All) => {
                *this = PermissionMethods::All;
            }
            (PermissionMethods::Some(set), PermissionMethods::Some(other_set)) => {
                set.extend(other_set);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ContractCallRequirement {
    pub(crate) contract: Option<String>,
    pub(crate) method: Option<String>,
}
