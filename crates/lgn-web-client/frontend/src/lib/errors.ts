/**
 * Takes an unknown error value and "stringify" it somehow.
 *
 * If the error value is of type `Error` then its message attribute is returned,
 * if it's a string then the value itself is returned, otherwise a default error
 * message is returned.
 *
 * The default error message can be specified.
 */
export function displayError(
  error: unknown,
  defaultMessage = "An unknown error occured"
) {
  return error instanceof Error
    ? error.message
    : typeof error === "string"
    ? error
    : defaultMessage;
}
