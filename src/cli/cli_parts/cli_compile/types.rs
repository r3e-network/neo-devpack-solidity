use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ManifestPermissionsMode {
    Merge,
    ReplaceWildcards,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ManifestPermissionMethods {
    All,
    Some(BTreeSet<String>),
}

impl ManifestPermissionMethods {
    pub(crate) fn merge_in(&mut self, other: ManifestPermissionMethods) {
        match (self, other) {
            (ManifestPermissionMethods::All, _) => {}
            (this, ManifestPermissionMethods::All) => {
                *this = ManifestPermissionMethods::All;
            }
            (ManifestPermissionMethods::Some(set), ManifestPermissionMethods::Some(other_set)) => {
                set.extend(other_set);
            }
        }
    }

    pub(crate) fn is_wildcard(&self) -> bool {
        matches!(self, ManifestPermissionMethods::All)
    }
}

pub(crate) type ManifestPermissionMap = BTreeMap<String, ManifestPermissionMethods>;

#[derive(Clone, Debug)]
pub(crate) struct ManifestPermissionsOverride {
    pub(crate) mode: ManifestPermissionsMode,
    pub(crate) permissions: ManifestPermissionMap,
}

#[derive(Clone, Debug)]
pub struct CompileOptions {
    pub optimizer_level: u8,
    pub use_callt: bool,
    pub deny_wildcard_permissions: bool,
    pub deny_wildcard_contracts: bool,
    pub deny_wildcard_methods: bool,
    pub(crate) manifest_permissions: Option<ManifestPermissionsOverride>,
}

impl CompileOptions {
    pub fn new(optimizer_level: u8, use_callt: bool) -> Self {
        Self {
            optimizer_level,
            use_callt,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            manifest_permissions: None,
        }
    }
}
