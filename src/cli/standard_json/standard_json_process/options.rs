use super::*;

#[derive(Clone, Debug)]
pub(crate) struct StandardJsonOptions<'a> {
    pub optimizer_level: u8,
    pub use_callt: bool,
    pub deny_wildcard_permissions: bool,
    pub deny_wildcard_contracts: bool,
    pub deny_wildcard_methods: bool,
    pub nef_source: Option<&'a str>,
    pub manifest_permissions: Option<ManifestPermissionsOverride>,
    pub contract_names: Vec<String>,
}
