import { Writable } from "../lib/store";

export default class Files extends Writable<File[] | null> {
  constructor() {
    super(null);
  }

  open({
    multiple,
    mimeTypes,
  }: { multiple?: boolean; mimeTypes?: string[] } = {}) {
    const fileInput = document.createElement("input");

    fileInput.type = "file";
    fileInput.multiple = !!multiple;
    fileInput.style.display = "none";

    const mimes = mimeTypes?.join(",");

    if (mimes) {
      fileInput.accept = mimes;
    }

    fileInput.addEventListener("change", (event) => {
      if (event.target instanceof HTMLInputElement) {
        // TODO: Use the `FileList` type for performance if needed
        this.set(event.target.files && Array.from(event.target.files));
      }
    });

    document.body.appendChild(fileInput);

    const event = new MouseEvent("click", {});

    fileInput.dispatchEvent(event);

    document.body.removeChild(fileInput);
  }
}
