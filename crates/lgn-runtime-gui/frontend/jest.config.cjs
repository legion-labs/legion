// @ts-check

module.exports = {
  testEnvironment: "jsDom",
  transform: {
    "^.+\\.svelte$": ["svelte-jester", { preprocess: true }],
    "^.+\\.(t|j)s$": "@swc/jest",
  },
  moduleFileExtensions: ["js", "ts", "svelte"],
  testMatch: ["**/tests/**/*.test.ts"],
  moduleNameMapper: { "^\\@\\/(.*)": "<rootDir>/src/$1" },
  setupFilesAfterEnv: ["<rootDir>/tests/setup.ts"],
  transformIgnorePatterns: ["node_modules/(?!@tauri-apps/api)"],
};
