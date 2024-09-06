<script>
  import { onMount } from "svelte";
  // import Quill from "quill";
  // import QuillMarkdown from "quilljs-markdown";
  // import { deltaToMarkdown } from "quill-delta-to-markdown";
  // import { MarkdownToQuill } from "md-to-quill-delta";

  onMount(async () => {
    const { default: Quill } = await import("quill");

    const quill = new Quill("#editor", {
      modules: {
        toolbar: false,
      },
      theme: "bubble",
      formats: [],
      placeholder: "Type Here!",
    });
  });
</script>

<div id="editor" />
