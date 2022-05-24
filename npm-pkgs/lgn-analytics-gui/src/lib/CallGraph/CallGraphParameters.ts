import { endQueryParam, startQueryParam } from "@/lib/time";

export class CallGraphParameters {
  processId: string;
  beginMs: number;
  endMs: number;

  constructor(processId: string, beginMs: number, endMs: number) {
    this.processId = processId;
    this.beginMs = beginMs;
    this.endMs = endMs;
  }

  static getGraphParameter(value: string) {
    const params = new URLSearchParams(value);

    const processId = params.get("process");
    if (processId === null || !processId.length) {
      throw new Error("missing param process");
    }

    const beginStr = params.get(startQueryParam);
    if (beginStr === null || !beginStr.length) {
      throw new Error("missing param begin");
    }

    const beginMs = parseFloat(beginStr);
    if (isNaN(beginMs)) {
      throw new Error(`param begin is not a valid float: ${beginStr}`);
    }

    const endStr = params.get(endQueryParam);
    if (endStr === null || !endStr.length) {
      throw new Error("missing param end");
    }

    const endMs = parseFloat(endStr);
    if (isNaN(endMs)) {
      throw new Error(`param end is not a valid float: ${endStr}`);
    }

    return new CallGraphParameters(processId, beginMs, endMs);
  }
}
