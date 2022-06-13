import "@testing-library/jest-dom";
import fetch from "node-fetch";

globalThis.fetch = fetch as typeof globalThis.fetch;
