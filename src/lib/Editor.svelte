<script lang="ts">
  import { onMount } from "svelte";
  import QuillMarkdown from "quilljs-markdown";
  import "quill/dist/quill.bubble.css";
  import "quilljs-markdown/dist/quilljs-markdown-common-style.css";

  onMount(async () => {
    const { default: Quill } = await import("quill");
    const { appWindow, PhysicalPosition, PhysicalSize } = await import(
      "@tauri-apps/api/window"
    );

    const quill = new Quill("#editor", {
      theme: "bubble",
      placeholder: "Empty Note",
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
