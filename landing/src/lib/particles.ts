export function initParticles(canvas: HTMLCanvasElement): () => void {
  const ctx = canvas.getContext("2d");
  if (!ctx) return () => {};

  let animId = 0;
  let particles: { x: number; y: number; vx: number; vy: number }[] = [];
  const COUNT = 200;
  const CONNECT_DIST = 120;

  function resize() {
    canvas.width = canvas.offsetWidth * devicePixelRatio;
    canvas.height = canvas.offsetHeight * devicePixelRatio;
    ctx!.scale(devicePixelRatio, devicePixelRatio);
  }
  resize();
  window.addEventListener("resize", resize);

  for (let i = 0; i < COUNT; i++) {
    particles.push({
      x: Math.random() * canvas.offsetWidth,
      y: Math.random() * canvas.offsetHeight,
      vx: (Math.random() - 0.5) * 0.3,
      vy: (Math.random() - 0.5) * 0.3,
    });
  }

  function frame() {
    const w = canvas.offsetWidth;
    const h = canvas.offsetHeight;
    ctx!.clearRect(0, 0, w, h);

    for (const p of particles) {
      p.x += p.vx;
      p.y += p.vy;
      if (p.x < 0 || p.x > w) p.vx *= -1;
      if (p.y < 0 || p.y > h) p.vy *= -1;
    }

    for (let i = 0; i < particles.length; i++) {
      for (let j = i + 1; j < particles.length; j++) {
        const dx = particles[i].x - particles[j].x;
        const dy = particles[i].y - particles[j].y;
        const dist = Math.sqrt(dx * dx + dy * dy);
        if (dist < CONNECT_DIST) {
          ctx!.beginPath();
          ctx!.moveTo(particles[i].x, particles[i].y);
          ctx!.lineTo(particles[j].x, particles[j].y);
          ctx!.strokeStyle = `rgba(0, 240, 255, ${(1 - dist / CONNECT_DIST) * 0.15})`;
          ctx!.lineWidth = 0.5;
          ctx!.stroke();
        }
      }
    }

    for (const p of particles) {
      ctx!.beginPath();
      ctx!.arc(p.x, p.y, 1, 0, Math.PI * 2);
      ctx!.fillStyle = "rgba(0, 240, 255, 0.3)";
      ctx!.fill();
    }

    animId = requestAnimationFrame(frame);
  }

  animId = requestAnimationFrame(frame);

  return () => {
    cancelAnimationFrame(animId);
    window.removeEventListener("resize", resize);
  };
}
