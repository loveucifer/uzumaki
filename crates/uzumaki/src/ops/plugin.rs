use std::str::FromStr;

use deno_core::*;

use crate::app::{SharedAppState, with_state};
use crate::plugin::{PluginCapability, PluginPermissionPolicyInfo};

#[op2]
#[serde]
pub fn op_list_plugins(state: &mut OpState) -> Vec<crate::plugin::PluginManifest> {
    let app_state = state.borrow::<SharedAppState>().clone();
    with_state(&app_state, |s| s.plugins.manifests())
}

#[op2(fast)]
pub fn op_has_plugin_capability(
    state: &mut OpState,
    #[string] capability: String,
) -> Result<bool, deno_error::JsErrorBox> {
    let parsed = PluginCapability::from_str(&capability).map_err(|err| {
        deno_error::JsErrorBox::new(
            "InvalidPluginCapability",
            format!("invalid plugin capability '{capability}': {err}"),
        )
    })?;

    let app_state = state.borrow::<SharedAppState>().clone();
    Ok(with_state(&app_state, |s| s.plugins.has_capability(parsed)))
}

#[op2]
#[serde]
pub fn op_get_plugin_policy(state: &mut OpState) -> PluginPermissionPolicyInfo {
    let app_state = state.borrow::<SharedAppState>().clone();
    with_state(&app_state, |s| s.plugins.policy_info())
}

#[op2]
#[serde]
pub fn op_list_denied_plugin_capabilities(state: &mut OpState) -> Vec<PluginCapability> {
    let app_state = state.borrow::<SharedAppState>().clone();
    with_state(&app_state, |s| s.plugins.denied_capabilities())
}

#[op2(fast)]
pub fn op_require_plugin_capability(
    state: &mut OpState,
    #[string] capability: String,
) -> Result<(), deno_error::JsErrorBox> {
    let parsed = PluginCapability::from_str(&capability).map_err(|err| {
        deno_error::JsErrorBox::new(
            "InvalidPluginCapability",
            format!("invalid plugin capability '{capability}': {err}"),
        )
    })?;

    let app_state = state.borrow::<SharedAppState>().clone();
    with_state(&app_state, |s| {
        s.plugins.require_capability(parsed).map_err(|err| {
            deno_error::JsErrorBox::new(
                "PluginCapabilityUnavailable",
                format!("capability '{capability}' is unavailable: {err}"),
            )
        })
    })
}