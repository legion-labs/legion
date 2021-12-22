export default {
  testEnvironment: "jsDom",
  transform: {
    "^.+\\.svelte$": [
      // Related issue and fix: https://github.com/mihar-22/svelte-jester/issues/72#issuecomment-982494278
      "<rootDir>/node_modules/svelte-jester/dist/transformer.mjs",
      {
        preprocess: true,
      },
    ],
    "^.+\\.(t|j)s$": "@swc/jest",
  },
  extensionsToTreatAsEsm: [".ts", ".svelte"],
  moduleFileExtensions: ["js", "ts", "svelte"],
  testMatch: ["**/tests/**/*.test.ts"],
  moduleNameMapper: {
    "^\\@\\/(.*)": "<rootDir>/src/$1",
  },
  setupFilesAfterEnv: ["<rootDir>/tests/setup.ts"],
  transformIgnorePatterns: ["node_modules/(?!@tauri-apps/api)"],
};
