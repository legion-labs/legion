import { endQueryParam, startQueryParam } from "@/lib/time";

export class GraphParameters {
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
    if (!processId) {
      throw new Error("missing param process");
    }
    const beginStr = params.get(startQueryParam);
    if (!beginStr) {
      throw new Error("missing param begin");
    }
    const endStr = params.get(endQueryParam);
    if (!endStr) {
      throw new Error("missing param end");
    }
    return new GraphParameters(
      processId,
      parseFloat(beginStr),
      parseFloat(endStr)
    );
  }
}
