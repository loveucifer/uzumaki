import core, {
  type PluginCapability,
  type PluginManifest,
  type PluginPolicyInfo,
} from './core';

export class Plugins {
  /**
   * Returns manifests for all loaded native plugins.
   * Useful for diagnostics and plugin compatibility checks.
   */
  static list(): PluginManifest[] {
    return core.listPlugins();
  }

  /**
   * Capability checks let apps fail early with clear errors instead of
   * crashing halfway through a feature flow.
   */
  static has(capability: PluginCapability): boolean {
    return core.hasPluginCapability(capability);
  }

  /**
   * Throws if a required capability is unavailable.
   *
   * This is useful for feature modules that should fail fast with a clear
   * message instead of triggering partial runtime behavior.
   */
  static require(capability: PluginCapability): void {
    core.requirePluginCapability(capability);
  }

  static policy(): PluginPolicyInfo {
    return core.getPluginPolicy();
  }

  static deniedCapabilities(): PluginCapability[] {
    return core.listDeniedPluginCapabilities();
  }
}

export type { PluginCapability, PluginManifest, PluginPolicyInfo };