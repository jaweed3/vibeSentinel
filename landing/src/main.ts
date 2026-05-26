import "./style.css";
import { initParticles } from "./lib/particles";
import { initScrollReveal, initMetrics } from "./lib/animate";
import { initTabs } from "./components/tabs";

document.addEventListener("DOMContentLoaded", () => {
  const canvas = document.getElementById("heroCanvas") as HTMLCanvasElement;
  if (canvas) initParticles(canvas);

  initScrollReveal(".tl-phase");
  initMetrics();
  initTabs();
});
