export function initTabs() {
  const tabs = document.querySelectorAll(".demo-tab");
  const panels = document.querySelectorAll(".demo-panel");

  tabs.forEach((tab) => {
    tab.addEventListener("click", () => {
      const target = tab.getAttribute("data-tab");
      if (!target) return;

      tabs.forEach((t) => t.classList.remove("active"));
      panels.forEach((p) => p.classList.remove("active"));

      tab.classList.add("active");
      const panel = document.querySelector(`[data-panel="${target}"]`);
      if (panel) panel.classList.add("active");
    });
  });
}
