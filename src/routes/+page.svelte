<script lang="ts">
  import Editor from "$lib/Editor.svelte";
  import { onMount } from "svelte";
  import "./page.css";

  onMount(async () => {
    const { appWindow } = await import("@tauri-apps/api/window");
    const { invoke } = await import("@tauri-apps/api/tauri");

    document.body.style.backgroundColor = "#fff9b1";

    async function fetchColors() {
      return (await invoke("get_colors")) as string[];
    }

    async function saveColor(selectedColor: string) {
      await invoke("add_color", { color: selectedColor });
    }

    document
      .getElementById("titlebar-close")
      ?.addEventListener("click", async () => {
        await invoke("remove_window", { label: appWindow.label });
        appWindow.close();
      });

    let colorMenuOpen = false;
    let colorPickerOpen = false;

    async function openColorMenu() {
      const target = document.getElementById(
        "titlebar-color"
      ) as HTMLDivElement;

      document.getElementById("titlebar")!.classList.add("hover");

      let colors = await fetchColors();

      // thank you stackOverFlow!
      const rgba2hex = (rgba: string) =>
        `#${(
          rgba.match(
            /^rgb?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*(\d+\.{0,1}\d*))?\)$/
          ) ?? []
        )
          .slice(1)
          .map((n, i) =>
            (i === 3 ? Math.round(parseFloat(n) * 255) : parseFloat(n))
              .toString(16)
              .padStart(2, "0")
              .replace("NaN", "")
          )
          .join("")}`;

      colors = colors.filter(
        (c) =>
          c.toLowerCase() !=
          rgba2hex(document.body.style.backgroundColor).toLowerCase()
      );

      colors.forEach((color) => {
        const colorBox = document.createElement("div");
        colorBox.className = "colorBox";
        colorBox.style.backgroundColor = color;

        colorBox.addEventListener("click", (e) => {
          document.body.style.backgroundColor = (
            e.target as HTMLDivElement
          ).style.backgroundColor;

          // Delay menu closing by a small amount to allow DOM changes to take effect
          setTimeout(() => closeColorMenu(), 0);
        });

        target.appendChild(colorBox);
      });

      const addButton = document.createElement("img");
      addButton.src = "https://api.iconify.design/mdi:add.svg";
      addButton.alt = "pick";
      addButton.addEventListener("click", (e) => {
        closeColorMenu();

        (
          document.getElementById("titlebar-color-icon") as HTMLImageElement
        ).src = "https://api.iconify.design/mdi:add.svg";

        colorPickerOpen = true;

        document.getElementById("titlebar")!.classList.add("hover");

        const textbox = document.createElement("input");
        textbox.id = "color-picker";
        textbox.type = "text";
        textbox.placeholder = "type a hex code";

        target.appendChild(textbox);

        e.stopPropagation();
      });
      target.appendChild(addButton);

      colorMenuOpen = true;
    }

    function closeColorMenu() {
      const target = document.getElementById(
        "titlebar-color"
      ) as HTMLDivElement;

      document.getElementById("titlebar")!.classList.remove("hover");

      const icon = document.getElementById("titlebar-color-icon");
      target.innerHTML = "";
      if (icon) target.appendChild(icon);
      colorMenuOpen = false;
    }

    async function pickColor() {
      const target = document.getElementById(
        "color-picker"
      ) as HTMLInputElement;

      // thanks chatgpt!
      const hexRegex =
        /^#([A-Fa-f0-9]{3}|[A-Fa-f0-9]{6}|[A-Fa-f0-9]{4}|[A-Fa-f0-9]{8})$/;
      if (hexRegex.test(target.value)) {
        saveColor(target.value);

        document.body.style.backgroundColor = target.value;
      }

      if (hexRegex.test(target.value) || target.value.length == 0) {
        document.getElementById("titlebar")!.classList.remove("hover");

        const icon = document.getElementById("titlebar-color-icon");
        const button = document.getElementById("titlebar-color");
        if (button) button.removeChild(target);
        (icon as HTMLImageElement).src =
          "https://api.iconify.design/mdi:color.svg";

        colorPickerOpen = false;
      }
    }

    document
      .getElementById("titlebar-color-icon")
      ?.addEventListener("click", async () => {
        if (colorMenuOpen && !colorPickerOpen) {
          closeColorMenu();
        } else if (colorPickerOpen) {
          pickColor();
        } else {
          openColorMenu();
        }
      });
  });
</script>

<div data-tauri-drag-region id="titlebar">
  <div class="titlebar-button" id="titlebar-close">
    <img src="https://api.iconify.design/mdi:close.svg" alt="close" />
  </div>
  <div class="titlebar-button" id="titlebar-color">
    <img
      src="https://api.iconify.design/mdi:color.svg"
      alt="color"
      id="titlebar-color-icon"
    />
  </div>
</div>

<Editor />
