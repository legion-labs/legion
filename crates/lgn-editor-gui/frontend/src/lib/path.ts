/** Dummy alias for string */
export type Path = string;

export type MainSeparator = "/" | "\\";

/**
 * Detects the main path separator / or \\
 * If both seperators are present in the string
 * the main separator will be assumed to be the first found
 */
export function detectMainPathSeparator(path: Path): MainSeparator | null {
  for (const c of path) {
    if (c === "/") {
      return "/";
    }

    if (c === "\\") {
      return "\\";
    }
  }

  return null;
}

/** Split a path into components. Empty parts are removed. */
export function components(path: Path): string[] {
  // We assume / as the path seperator if none detected
  const pathSeparator = detectMainPathSeparator(path) || "/";

  return path.split(pathSeparator).filter(Boolean);
}

/** Extract the file name from a path */
export function fileName(path: Path): string | null {
  const parts = components(path);

  if (!parts.length) {
    return null;
  }

  return parts[parts.length - 1];
}

/** Extract the extension from a path */
export function extension(path: Path): string | null {
  const pathFileName = fileName(path);

  if (!pathFileName) {
    return null;
  }

  const pathFileNameParts = pathFileName.split(".").filter(Boolean);

  // Having only 0 or 1 part means there was no `.` found (and therefore no extensions)
  // _or_ that the file name starts with a `.`
  if (pathFileNameParts.length <= 1) {
    return null;
  }

  return pathFileNameParts.reverse()[0];
}

/** Joins the path compenents into a `Path` using the provided main separator */
export function absolute(
  components: string[],
  mainSeparator: MainSeparator
): Path {
  const cleanComponents = components.filter(Boolean);

  if (!cleanComponents.length) {
    return `${mainSeparator}`;
  }

  return `${mainSeparator}${cleanComponents.join(mainSeparator)}`;
}
