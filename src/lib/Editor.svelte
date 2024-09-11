<script lang="ts">
  import { onMount } from "svelte";
  import QuillMarkdown from "quilljs-markdown";
  import "quill/dist/quill.bubble.css";
  import "quilljs-markdown/dist/quilljs-markdown-common-style.css";

  onMount(async () => {
    const { default: Quill, Range } = await import("quill");
    const { Delta } = await import("quill/core");

    const { appWindow, PhysicalPosition, PhysicalSize, LogicalSize } =
      await import("@tauri-apps/api/window");
    const { writeText, readText } = await import("@tauri-apps/api/clipboard");

    const quill = new Quill("#editor", {
      theme: "bubble",
      placeholder: "Empty Note",
    });

    quill.clipboard.addMatcher(Node.ELEMENT_NODE, function (node, delta) {
      var plaintext = node.textContent;
      if (plaintext) {
        return new Delta().insert(plaintext);
      } else {
        return new Delta();
      }
    });

    quill.on("text-change", async () => {
      let editor = document.querySelector(".ql-editor");

      const factor = await appWindow.scaleFactor();

      const window = (await appWindow.innerSize()).toLogical(factor);

      if (editor!.clientHeight + 20 + 12 > window.height) {
        appWindow.setSize(
          // 25 to get rid of the scroll bar
          new LogicalSize(window.width, editor!.clientHeight + 25)
        );
      }
    });

    const markdownOptions = {};

    // markdown is enabled
    const quillMarkdown = new QuillMarkdown(quill, markdownOptions);

    document.getElementById("editor")?.addEventListener("click", () => {
      quill.focus();
    });

    type InitPayload = {
      contents: string;
      color: string;
      x: number;
      y: number;
      height: number;
      width: number;
      label: string;
    };

    appWindow.listen("copy", () => {
      const selection = quill.getSelection();

      if (selection) {
        writeText(quill.getText(selection));
      }
    });

    appWindow.listen("cut", async () => {
      const selection = quill.getSelection();

      const text = await readText();

      if (selection && text) {
        writeText(quill.getText(selection));
        quill.deleteText(selection);
      }
    });

    appWindow.listen("paste", async () => {
      const selection = quill.getSelection();

      const text = await readText();

      if (selection && text) {
        quill.deleteText(selection);
        quill.insertText(selection.index, text);
      }
    });

    appWindow.listen("select_all", () => {
      setTimeout(
        () => quill.setSelection(new Range(0, quill.getText().length)),
        0
      );
    });

    appWindow.listen("save-contents-request", async () => {
      const pos = await appWindow.outerPosition();
      const size = await appWindow.innerSize();
      appWindow.emit("save-contents-response", {
        contents: JSON.stringify(quill.getContents()),
        label: appWindow.label,
        color: document.body.style.backgroundColor,
        x: pos.x,
        y: pos.y,
        width: size.width,
        height: size.height,
      });
    });

    appWindow.listen("init", (event) => {
      const payload = event.payload as InitPayload;

      quill.setContents(JSON.parse(payload.contents));
      document.body.style.backgroundColor = payload.color;

      appWindow.setPosition(new PhysicalPosition(payload.x, payload.y));

      appWindow.setSize(new PhysicalSize(payload.width, payload.height));
    });

    appWindow.listen("fit_text", async () => {
      let editor = document.querySelector(".ql-editor");

      const factor = await appWindow.scaleFactor();

      const window = (await appWindow.innerSize()).toLogical(factor);

      appWindow.setSize(
        // 25 to get rid of the scroll bar
        new LogicalSize(window.width, editor!.clientHeight + 25)
      );
    });

    if (appWindow.label != "main") appWindow.show();

    // not sure why, but this glitches out the cursor in the editor
    setTimeout(() => {
      quill.focus();
    }, 100);

    appWindow.emit("ready", {});
  });
</script>

<div id="editor" />

<style>
  #editor {
    width: 100%;
    height: 100%;
  }
</style>
