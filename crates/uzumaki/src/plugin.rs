use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Capabilities a plugin may request.
///
/// Keep these values stable once released because they are part of the public
/// plugin contract and are referenced by app-level permission policies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PluginCapability {
    Camera,
    Bluetooth,
    FilesystemWatch,
    MediaDecode,
    MediaPlayback,
    Notifications,
    Tray,
    NativeMenu,
    Webview,
    GpuSharedTextures,
}

impl PluginCapability {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Camera => "camera",
            Self::Bluetooth => "bluetooth",
            Self::FilesystemWatch => "filesystemWatch",
            Self::MediaDecode => "mediaDecode",
            Self::MediaPlayback => "mediaPlayback",
            Self::Notifications => "notifications",
            Self::Tray => "tray",
            Self::NativeMenu => "nativeMenu",
            Self::Webview => "webview",
            Self::GpuSharedTextures => "gpuSharedTextures",
        }
    }
}

impl FromStr for PluginCapability {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "camera" => Ok(Self::Camera),
            "bluetooth" => Ok(Self::Bluetooth),
            "filesystemWatch" => Ok(Self::FilesystemWatch),
            "mediaDecode" => Ok(Self::MediaDecode),
            "mediaPlayback" => Ok(Self::MediaPlayback),
            "notifications" => Ok(Self::Notifications),
            "tray" => Ok(Self::Tray),
            "nativeMenu" => Ok(Self::NativeMenu),
            "webview" => Ok(Self::Webview),
            "gpuSharedTextures" => Ok(Self::GpuSharedTextures),
            _ => Err(format!("unknown plugin capability: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<PluginCapability>,
}

#[derive(Debug, Clone)]
pub struct PluginLifecycleContext {
    pub app_root: std::path::PathBuf,
    pub entrypoint: std::path::PathBuf,
}

pub trait NativePlugin {
    fn manifest(&self) -> &PluginManifest;

    /// Called once when the runtime starts.
    fn on_runtime_start(&mut self, _ctx: &PluginLifecycleContext) -> anyhow::Result<()> {
        Ok(())
    }

    /// Called before the runtime shuts down.
    fn on_runtime_stop(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginPermissionPolicyInfo {
    pub source: String,
    pub allow: Vec<PluginCapability>,
    pub deny: Vec<PluginCapability>,
}

#[derive(Debug, Clone)]
pub struct PluginPermissionPolicy {
    source: String,
    allow: Option<HashSet<PluginCapability>>,
    deny: HashSet<PluginCapability>,
}

impl PluginPermissionPolicy {
    pub fn allow_all() -> Self {
        Self {
            source: "default".to_string(),
            allow: None,
            deny: HashSet::new(),
        }
    }

    pub fn from_app_root(app_root: &Path) -> Self {
        let path = app_root.join("uzumaki.config.json");
        if !path.exists() {
            return Self::allow_all();
        }

        let Ok(raw) = fs::read_to_string(&path) else {
            eprintln!("[uzumaki][plugin] failed to read {}", path.display());
            return Self::allow_all();
        };

        let Ok(parsed) = serde_json::from_str::<UzumakiPluginPolicyConfig>(&raw) else {
            eprintln!("[uzumaki][plugin] failed to parse {}", path.display());
            return Self::allow_all();
        };

        let Some(plugin_cfg) = parsed.plugins else {
            return Self::allow_all();
        };

        let mut allow_set = HashSet::new();
        for name in plugin_cfg.allow {
            match PluginCapability::from_str(&name) {
                Ok(cap) => {
                    allow_set.insert(cap);
                }
                Err(_) => {
                    eprintln!("[uzumaki][plugin] unknown capability in allow list: {name}");
                }
            }
        }

        let mut deny_set = HashSet::new();
        for name in plugin_cfg.deny {
            match PluginCapability::from_str(&name) {
                Ok(cap) => {
                    deny_set.insert(cap);
                }
                Err(_) => {
                    eprintln!("[uzumaki][plugin] unknown capability in deny list: {name}");
                }
            }
        }

        // Deny wins when both are present.
        if !allow_set.is_empty() {
            for cap in &deny_set {
                allow_set.remove(cap);
            }
        }

        Self {
            source: path.display().to_string(),
            allow: if allow_set.is_empty() {
                None
            } else {
                Some(allow_set)
            },
            deny: deny_set,
        }
    }

    pub fn is_allowed(&self, capability: PluginCapability) -> bool {
        if self.deny.contains(&capability) {
            return false;
        }
        self.allow
            .as_ref()
            .map(|set| set.contains(&capability))
            .unwrap_or(true)
    }

    pub fn info(&self) -> PluginPermissionPolicyInfo {
        let mut allow: Vec<PluginCapability> = self
            .allow
            .as_ref()
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default();
        let mut deny: Vec<PluginCapability> = self.deny.iter().copied().collect();
        allow.sort_by_key(|cap| cap.as_str());
        deny.sort_by_key(|cap| cap.as_str());
        PluginPermissionPolicyInfo {
            source: self.source.clone(),
            allow,
            deny,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UzumakiPluginPolicyConfig {
    #[serde(default)]
    plugins: Option<PluginPolicySection>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PluginPolicySection {
    #[serde(default)]
    allow: Vec<String>,
    #[serde(default)]
    deny: Vec<String>,
}

pub struct PluginRegistry {
    plugins: Vec<Box<dyn NativePlugin>>,
    plugin_indices_by_name: HashMap<String, usize>,
    granted_caps: HashSet<PluginCapability>,
    denied_caps: HashSet<PluginCapability>,
    policy: PluginPermissionPolicy,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self {
            plugins: Vec::new(),
            plugin_indices_by_name: HashMap::new(),
            granted_caps: HashSet::new(),
            denied_caps: HashSet::new(),
            policy: PluginPermissionPolicy::allow_all(),
        }
    }
}

impl PluginRegistry {
    pub fn new(policy: PluginPermissionPolicy) -> Self {
        Self {
            policy,
            ..Self::default()
        }
    }

    pub fn register(&mut self, plugin: Box<dyn NativePlugin>) -> anyhow::Result<()> {
        let manifest = plugin.manifest();
        if manifest.name.trim().is_empty() {
            anyhow::bail!("plugin name cannot be empty");
        }
        if self.plugin_indices_by_name.contains_key(&manifest.name) {
            anyhow::bail!("plugin '{}' already registered", manifest.name);
        }

        for cap in &manifest.capabilities {
            if self.policy.is_allowed(*cap) {
                self.granted_caps.insert(*cap);
            } else {
                self.denied_caps.insert(*cap);
                anyhow::bail!(
                    "plugin '{}' requested denied capability '{}'; update plugins.allow/deny in uzumaki.config.json",
                    manifest.name,
                    cap.as_str()
                );
            }
        }

        self.plugin_indices_by_name
            .insert(manifest.name.clone(), self.plugins.len());
        self.plugins.push(plugin);
        Ok(())
    }

    pub fn manifests(&self) -> Vec<PluginManifest> {
        self.plugins
            .iter()
            .map(|plugin| plugin.manifest().clone())
            .collect()
    }

    pub fn has_capability(&self, capability: PluginCapability) -> bool {
        self.granted_caps.contains(&capability)
    }

    pub fn denied_capabilities(&self) -> Vec<PluginCapability> {
        let mut denied: Vec<PluginCapability> = self.denied_caps.iter().copied().collect();
        denied.sort_by_key(|cap| cap.as_str());
        denied
    }

    pub fn require_capability(&self, capability: PluginCapability) -> anyhow::Result<()> {
        if self.has_capability(capability) {
            return Ok(());
        }

        let denied = self.denied_caps.contains(&capability);
        let reason = if denied {
            "denied by policy"
        } else {
            "no loaded plugin provides this capability"
        };

        anyhow::bail!(
            "required capability '{}' unavailable: {reason}",
            capability.as_str()
        )
    }

    pub fn policy_info(&self) -> PluginPermissionPolicyInfo {
        self.policy.info()
    }

    pub fn on_runtime_start(&mut self, ctx: &PluginLifecycleContext) {
        for plugin in &mut self.plugins {
            if let Err(err) = plugin.on_runtime_start(ctx) {
                eprintln!(
                    "[uzumaki][plugin:{}] start hook failed: {err}",
                    plugin.manifest().name
                );
            }
        }
    }

    pub fn on_runtime_stop(&mut self) {
        for plugin in &mut self.plugins {
            if let Err(err) = plugin.on_runtime_stop() {
                eprintln!(
                    "[uzumaki][plugin:{}] stop hook failed: {err}",
                    plugin.manifest().name
                );
            }
        }
    }
}

/// Minimal built-in plugin so the registry is never empty and clients can
/// verify lifecycle wiring in development.
struct CoreRuntimePlugin {
    manifest: PluginManifest,
}

impl CoreRuntimePlugin {
    fn new() -> Self {
        Self {
            manifest: PluginManifest {
                name: "uzumaki.core".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                capabilities: vec![],
            },
        }
    }
}

impl NativePlugin for CoreRuntimePlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }
}

pub fn register_builtin_plugins(registry: &mut PluginRegistry) {
    if let Err(err) = registry.register(Box::new(CoreRuntimePlugin::new())) {
        eprintln!("[uzumaki][plugin] failed to register built-in plugin: {err}");
    }
}