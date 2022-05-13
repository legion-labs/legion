import { UserInfo } from "@lgn/auth";

declare global {
  // eslint-disable-next-line no-var
  var isElectron: boolean | undefined;

  // eslint-disable-next-line no-var
  var electron:
    | {
        toggleMaximizeMainWindow(this: void): void;
        minimizeMainWindow(this: void): void;
        closeMainWindow(this: void): void;
        auth: {
          initOAuthClient(this: void): Promise<void>;
          authenticate(
            scopes: string[],
            extraParams?: Record<string, string> | null | undefined
          ): Promise<UserInfo>;
          getAccessToken(this: void): string;
        };
      }
    | undefined;
}
