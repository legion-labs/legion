// Contains all the scripts opened in tabs
import type { Writable } from "svelte/store";
import { writable } from "svelte/store";

import type { ServerType } from "@lgn/web-client/src/api";

export type ScriptTabTypePayload = {
  type: "script";
  readonly?: boolean;
  lang: string;
  value: string;
};

export type VideoTabTypePayload = {
  type: "video";
  serverType: ServerType;
};

export type SceneExplorerTypePayload = {
  type: "sceneExplorer";
  rootSceneId: string;
};

export type TabPayload =
  | ScriptTabTypePayload
  | VideoTabTypePayload
  | SceneExplorerTypePayload;

export type TabPayloadsValue = Record<string, TabPayload>;

export type TabPayloadsStore = Writable<TabPayloadsValue>;

export default writable<TabPayloadsValue>({});
