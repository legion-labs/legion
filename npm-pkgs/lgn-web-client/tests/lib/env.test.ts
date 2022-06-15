import { getEnv } from "../../src/lib/env";

const allowedApp = "next-big-thing";
const allowedDomain = "great.com";

describe("getEnv", () => {
  test("return local when the URL is localhost", () => {
    expect(
      getEnv({
        allowedApp,
        allowedDomain,
        url: new URL("http://localhost:3000"),
      })
    ).toEqual("local");
  });

  test("return null when the URL is not allowed", () => {
    expect(
      getEnv({
        allowedApp,
        allowedDomain,
        url: new URL("http://invalid.com"),
      })
    ).toEqual(null);
  });

  test("return null when the app is valid, but not the domain", () => {
    expect(
      getEnv({
        allowedApp,
        allowedDomain,
        url: new URL(`http://${allowedApp}.invalid.com`),
      })
    ).toEqual(null);
  });

  test("return null when the domain is valid, but not the app", () => {
    expect(
      getEnv({
        allowedApp,
        allowedDomain,
        url: new URL(`http://invalid.${allowedDomain}`),
      })
    ).toEqual(null);
  });

  test("return null when the app and the domain are valid, but the format is not", () => {
    expect(
      getEnv({
        allowedApp,
        allowedDomain,
        url: new URL(`http://${allowedDomain}.${allowedApp}`),
      })
    ).toEqual(null);
  });

  test("return production when the app and the domain are valid, and no env are provided", () => {
    expect(
      getEnv({
        allowedApp,
        allowedDomain,
        url: new URL(`http://${allowedApp}.${allowedDomain}`),
      })
    ).toEqual("production");
  });

  test("return production when the app and the domain are valid, and no env are provided, with path", () => {
    expect(
      getEnv({
        allowedApp,
        allowedDomain,
        url: new URL(`http://${allowedApp}.${allowedDomain}/something`),
      })
    ).toEqual("production");
  });

  test("return production when the app and the domain are valid, and no env are provided, with port", () => {
    expect(
      getEnv({
        allowedApp,
        allowedDomain,
        url: new URL(`http://${allowedApp}.${allowedDomain}:9999`),
      })
    ).toEqual("production");
  });

  test("return production when the app and the domain are valid, and no env are provided, with port and path", () => {
    expect(
      getEnv({
        allowedApp,
        allowedDomain,
        url: new URL(`http://${allowedApp}.${allowedDomain}:9999/something`),
      })
    ).toEqual("production");
  });

  test("return uat when the app and the domain are valid, and uat is in the url", () => {
    expect(
      getEnv({
        allowedApp,
        allowedDomain,
        url: new URL(`http://${allowedApp}.uat.${allowedDomain}`),
      })
    ).toEqual("uat");
  });

  test("return null when the app and the domain are valid, and uat is in the url, but the format is invalid", () => {
    expect(
      getEnv({
        allowedApp,
        allowedDomain,
        url: new URL(`http://uat.${allowedApp}.${allowedDomain}`),
      })
    ).toEqual(null);
  });

  test("return null when the app and the domain are valid, an env is provided in the url, but it's not valid", () => {
    expect(
      getEnv({
        allowedApp,
        allowedDomain,
        url: new URL(`http://${allowedApp}.invalid.${allowedDomain}`),
      })
    ).toEqual(null);
  });
});
