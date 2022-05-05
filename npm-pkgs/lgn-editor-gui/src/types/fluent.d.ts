/** A full dictionnary of all the found message ids, attributes, and variables */
declare type Fluent = {
  "hello-user": {
    attributes: null;
    variables: "userName";
  };

  "shared-photos": {
    attributes: null;
    variables: "userName" | "photoCount" | "userGender";
  };
};
