<script>
  import { onMount } from "svelte";
  import QuillMarkdown from "quilljs-markdown";
  import "quill/dist/quill.bubble.css";
  import "quilljs-markdown/dist/quilljs-markdown-common-style.css";

  onMount(async () => {
    const { default: Quill } = await import("quill");

    const quill = new Quill("#editor", {
      theme: "bubble",
      placeholder: "Empty Note",
    });

    const markdownOptions = {};

    // markdown is enabled
    const quillMarkdown = new QuillMarkdown(quill, markdownOptions);

    quill.focus();
  });
</script>

<div id="editor" />

<style>
  #editor {
    width: 100%;
    height: 100%;
  }
</style>
