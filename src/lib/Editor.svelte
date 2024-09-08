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
    const { invoke } = await import("@tauri-apps/api/tauri");

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
    };

    appWindow.listen("init", (event) => {
      const payload = event.payload as InitPayload;

      console.log("payload", payload);

      console.log(JSON.parse(payload.contents));

      quill.setContents(JSON.parse(payload.contents));
      document.body.style.backgroundColor = payload.color;

      appWindow.setPosition(new PhysicalPosition(payload.x, payload.y));

      appWindow.setSize(new PhysicalSize(payload.width, payload.height));
    });

    const saveContents = async () => {
      // console.log(appWindow.label);
      const pos = await appWindow.outerPosition();
      const size = await appWindow.innerSize();
      invoke("save_contents", {
        contents: JSON.stringify(quill.getContents()),
        label: appWindow.label,
        color: document.body.style.backgroundColor,
        x: pos.x,
        y: pos.y,
        width: size.width,
        height: size.height,
      });
    };

    let saveInterval: number | null = null;

    appWindow.onFocusChanged(({ payload: focused }) => {
      console.log("focus: ", appWindow.label, focused);
      if (focused) {
        saveInterval = setInterval(saveContents, 1000);
      } else {
        saveContents();
        if (saveInterval) {
          clearInterval(saveInterval);
        }
      }
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
