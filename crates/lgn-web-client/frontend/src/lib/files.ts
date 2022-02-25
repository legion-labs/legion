import { Observable } from "rxjs";

export function readFile(blob: Blob): Promise<ArrayBuffer> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();

    let result: ArrayBuffer | null = null;

    reader.onerror = (error) => reject(error);
    reader.onabort = (error) => reject(error);
    reader.onloadend = () =>
      result ? resolve(result) : reject("Result not loaded yett");

    reader.onload = () => {
      result = reader.result as ArrayBuffer;
    };

    return reader.readAsArrayBuffer(blob);
  });
}

export function readFileStream(blob: Blob): Observable</* u8 */ number> {
  return new Observable((subscribe) => {
    const reader = new FileReader();

    reader.onerror = (error) => subscribe.error(error);
    reader.onabort = (error) => subscribe.error(error);
    reader.onloadend = () => subscribe.complete();

    reader.onload = () => {
      // TODO: Compare this solution with `ArrayBuffer` slicing
      const dataView = new DataView(reader.result as ArrayBuffer);

      for (let i = 0; i < dataView.byteLength; i++) {
        subscribe.next(dataView.getUint8(i));
      }
    };

    return reader.readAsArrayBuffer(blob);
  });
}
