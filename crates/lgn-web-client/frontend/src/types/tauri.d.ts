import { UserInfo } from "../lib/auth";

declare module "@tauri-apps/api" {
  type Command<T extends string> = `plugin:browser|${T}`;

  function invoke(
    command: Command<"authenticate">,
    params: { scopes: string[]; extraParams?: Record<string, string> }
  ): Promise<UserInfo>;
  function invoke(command: Command<"get_access_token">): Promise<string>;
}
