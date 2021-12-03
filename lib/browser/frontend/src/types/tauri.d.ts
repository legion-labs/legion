import { UserInfo } from "@lgn/browser-auth";

declare module "@tauri-apps/api" {
  type Command<T extends string> = `plugin:browser|${T}`;

  function invoke(command: Command<"authenticate">): Promise<UserInfo>;
  function invoke(command: Command<"get_access_token">): Promise<string>;
}
