import {
  components,
  detectMainPathSeparator,
  extension,
  fileName,
} from "@/lib/path";

describe("detectMainPathSeparator", () => {
  test("detects the path separator for a given path", () => {
    expect(detectMainPathSeparator("")).toBe(null);
    expect(detectMainPathSeparator("foobar")).toBe(null);
    expect(detectMainPathSeparator(".foo")).toBe(null);
    expect(detectMainPathSeparator("/foobar")).toBe("/");
    expect(detectMainPathSeparator("foo/bar")).toBe("/");
    expect(detectMainPathSeparator("C:\\foobar")).toBe("\\");
    expect(detectMainPathSeparator("foo\\bar")).toBe("\\");
    expect(detectMainPathSeparator("C:\\foobar/baz")).toBe("\\");
    expect(detectMainPathSeparator("foo/bar\\baz")).toBe("/");
  });
});

describe("components", () => {
  test("returns the components of a path", () => {
    expect(components("")).toEqual([]);
    expect(components("foobar")).toEqual(["foobar"]);
    expect(components(".foo")).toEqual([".foo"]);
    expect(components("/foobar")).toEqual(["foobar"]);
    expect(components("foo/bar")).toEqual(["foo", "bar"]);
    expect(components("foo/bar/baz")).toEqual(["foo", "bar", "baz"]);
    expect(components("C:\\foobar")).toEqual(["C:", "foobar"]);
    expect(components("C:\\foobar\\baz")).toEqual(["C:", "foobar", "baz"]);
    expect(components("C:\\foo/bar")).toEqual(["C:", "foo/bar"]);
  });
});

describe("fileName", () => {
  test("detects the file name for a given path", () => {
    expect(fileName("")).toBe(null);
    expect(fileName("foobar")).toBe("foobar");
    expect(fileName(".foo")).toBe(".foo");
    expect(fileName("/foobar")).toBe("foobar");
    expect(fileName("foo/bar")).toBe("bar");
    expect(fileName("foo/bar/baz")).toBe("baz");
    expect(fileName("C:\\foobar")).toBe("foobar");
    expect(fileName("C:\\foobar\\baz")).toBe("baz");
    expect(fileName("C:\\foo/bar")).toBe("foo/bar");
  });
});

describe("extension", () => {
  test("extracts the extension of a file path", () => {
    expect(extension("foo.ts")).toBe("ts");
    expect(extension("foo")).toBe(null);
    expect(extension("")).toBe(null);
    expect(extension(".ts")).toBe(null);
    expect(extension(".foo.ts")).toBe("ts");
    expect(extension("foo.bar.ts")).toBe("ts");
    expect(extension(".")).toBe(null);
  });

  test("extracts the extension of a file path for relative paths", () => {
    expect(extension("./foo.ts")).toBe("ts");
    expect(extension("./foo")).toBe(null);
    expect(extension("./")).toBe(null);
    expect(extension("./.ts")).toBe(null);
    expect(extension("./.foo.ts")).toBe("ts");
    expect(extension("./foo.bar.ts")).toBe("ts");
    expect(extension("./.")).toBe(null);
  });

  test("extracts the extension of a file path for long relative paths", () => {
    expect(extension("../../../foo.ts")).toBe("ts");
    expect(extension("../../../foo")).toBe(null);
    expect(extension("../../../")).toBe(null);
    expect(extension("../../../.ts")).toBe(null);
    expect(extension("../../../.foo.ts")).toBe("ts");
    expect(extension("../../../foo.bar.ts")).toBe("ts");
    expect(extension("../../../.")).toBe(null);
  });
});
