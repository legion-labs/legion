declare module "@tauri-apps/api" {
  // The `UserInfo` struct returned by the `authenticate` function
  type UserInfo = {
    sub: string;
  };

  function invoke(command: "authenticate"): Promise<UserInfo>;
}
