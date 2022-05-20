/**
 * A full dictionnary of all the found message ids, attributes, and variables
 */
declare type Fluent = {
  "hello-user": {
    variables: "userName";
  };
  "shared-photos": {
    variables: "userName" | "photoCount" | "userGender";
  };
};
