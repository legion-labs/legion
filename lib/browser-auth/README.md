## Auth Browser

Simple auth library that can be used in browsers/Tauri.

This library exposes some function that will help you deal with Cognito and authenticate a user to your frontend application.

### Initialization:

### Workflow:

1. Get an auth code from Cognito:

Calling the `getAuthorizationCodeInteractive` will automatically redirects the user to Cognito which will in turn redirects them to the application home with a `code` appended to the url query params. At that point the user is _not_ authenticated.

2. Finalize authentication:

Now you got a code from Cognito you can use the `finalizeAwsCognitoAuth` function to fully authenticate the user. After the function is called some cookies will be populated in the browser and the user is now authenticated.

3. Get user info when needed:

Using the 2 functions `getAccessToken` and `getUserInfo` you can get the user's information:

```typescript
const userInfo = await getUserInfo(getAccessToken());

console.log(`User's email address: ${userInfo.email}`);
```
