export function animateCounter(el: HTMLElement, target: number, duration = 1200) {
  const start = performance.now();

  function tick(now: number) {
    const t = Math.min((now - start) / duration, 1);
    const eased = 1 - Math.pow(1 - t, 3);
    const val = Math.round(eased * target);
    el.textContent = val.toLocaleString();
    if (t < 1) requestAnimationFrame(tick);
  }

  requestAnimationFrame(tick);
}

export function initScrollReveal(selector: string) {
  const els = document.querySelectorAll(selector);
  const obs = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting) {
          entry.target.classList.add("visible");
        }
      }
    },
    { threshold: 0.15 }
  );
  for (const el of els) obs.observe(el);
  return () => obs.disconnect();
}

export function initMetrics() {
  const metrics = document.querySelectorAll(".metric");
  const obs = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting) {
          const num = entry.target.querySelector(".metric-num") as HTMLElement;
          const target = parseInt(entry.target.getAttribute("data-target") || "0");
          if (num && target) animateCounter(num, target);
          obs.unobserve(entry.target);
        }
      }
    },
    { threshold: 0.5 }
  );
  for (const m of metrics) obs.observe(m);
}
