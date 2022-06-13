import type { Writable } from "svelte/store";

import { DefaultSessionStorage } from "../lib/storage";
import { connected } from "../lib/store";

export type DevSettingsValue = {
  grpcEditorServerUrl: string;
  grpcRuntimeServerUrl: string;
  restEditorServerUrl: string;
  restRuntimeServerUrl: string;
};

export type DevSettingsStore = Writable<DevSettingsValue>;

export function createDevSettingsStore(
  key: string,
  defaultValue: DevSettingsValue
): DevSettingsStore {
  return connected<string, DevSettingsValue>(
    new DefaultSessionStorage(),
    key,
    defaultValue
  );
}
