declare module "@tauri-apps/api" {
  // The `UserInfo` struct returned by the `authenticate` function
  type UserInfo = {
    sub: string;
  };

  type Command<T extends string> = `plugin:browser|${T}`;

  function invoke(command: Command<"authenticate">): Promise<UserInfo>;
  function invoke(command: Command<"get_access_token">): Promise<string>;
}
