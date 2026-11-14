(() => {
  const trigger = document.querySelector("#architecture-zoom-trigger");
  const dialog = document.querySelector("#architecture-zoom-dialog");
  const viewport = document.querySelector("#architecture-zoom-viewport");
  const image = document.querySelector("#architecture-zoom-image");
  const level = document.querySelector("#architecture-zoom-level");
  if (!trigger || !dialog || !viewport || !image || !level) return;

  let fitScale = 1;
  let scale = 1;
  let panX = 0;
  let panY = 0;
  let pointer = null;

  const imageSize = () => ({
    width: Number(image.getAttribute("width")) || image.naturalWidth || 2020,
    height: Number(image.getAttribute("height")) || image.naturalHeight || 960,
  });

  const clampPan = () => {
    const { width, height } = imageSize();
    const maxX = Math.max(0, (width * scale - viewport.clientWidth) / 2);
    const maxY = Math.max(0, (height * scale - viewport.clientHeight) / 2);
    panX = Math.max(-maxX, Math.min(maxX, panX));
    panY = Math.max(-maxY, Math.min(maxY, panY));
  };

  const render = () => {
    const { width, height } = imageSize();
    clampPan();
    viewport.classList.toggle("is-pannable", width * scale > viewport.clientWidth || height * scale > viewport.clientHeight);
    image.style.width = `${width * scale}px`;
    image.style.height = `${height * scale}px`;
    image.style.left = `${(viewport.clientWidth - width * scale) / 2 + panX}px`;
    image.style.top = `${(viewport.clientHeight - height * scale) / 2 + panY}px`;
    level.value = `${Math.round((scale / fitScale) * 100)}%`;
  };

  const fit = () => {
    const { width, height } = imageSize();
    fitScale = Math.min(viewport.clientWidth / width, viewport.clientHeight / height) * 0.96;
    scale = fitScale;
    panX = 0;
    panY = 0;
    render();
  };

  const zoomAt = (factor, clientX, clientY) => {
    const next = Math.max(fitScale * 0.5, Math.min(fitScale * 6, scale * factor));
    if (next === scale) return;
    const { width, height } = imageSize();
    const bounds = viewport.getBoundingClientRect();
    const x = clientX - bounds.left;
    const y = clientY - bounds.top;
    const oldLeft = (viewport.clientWidth - width * scale) / 2 + panX;
    const oldTop = (viewport.clientHeight - height * scale) / 2 + panY;
    const imageX = (x - oldLeft) / scale;
    const imageY = (y - oldTop) / scale;
    scale = next;
    panX = x - imageX * scale - (viewport.clientWidth - width * scale) / 2;
    panY = y - imageY * scale - (viewport.clientHeight - height * scale) / 2;
    render();
  };

  const zoomCenter = (factor) => {
    const bounds = viewport.getBoundingClientRect();
    zoomAt(factor, bounds.left + bounds.width / 2, bounds.top + bounds.height / 2);
  };

  trigger.addEventListener("click", () => {
    dialog.showModal();
    requestAnimationFrame(() => {
      fit();
      viewport.focus();
    });
  });
  dialog.querySelector("[data-zoom-close]").addEventListener("click", () => dialog.close());
  dialog.querySelector("[data-zoom-reset]").addEventListener("click", fit);
  dialog.querySelector("[data-zoom-in]").addEventListener("click", () => zoomCenter(1.25));
  dialog.querySelector("[data-zoom-out]").addEventListener("click", () => zoomCenter(0.8));
  dialog.addEventListener("close", () => trigger.focus());
  dialog.addEventListener("click", (event) => {
    if (event.target === dialog) dialog.close();
  });

  viewport.addEventListener("wheel", (event) => {
    event.preventDefault();
    zoomAt(Math.exp(-event.deltaY * 0.0015), event.clientX, event.clientY);
  }, { passive: false });

  viewport.addEventListener("pointerdown", (event) => {
    if (event.button !== 0 || !viewport.classList.contains("is-pannable")) return;
    const bounds = image.getBoundingClientRect();
    if (event.clientX < bounds.left || event.clientX > bounds.right ||
        event.clientY < bounds.top || event.clientY > bounds.bottom) return;
    pointer = { id: event.pointerId, x: event.clientX, y: event.clientY };
    viewport.setPointerCapture(event.pointerId);
    viewport.classList.add("is-dragging");
  });
  viewport.addEventListener("pointermove", (event) => {
    if (!pointer || event.pointerId !== pointer.id) return;
    panX += event.clientX - pointer.x;
    panY += event.clientY - pointer.y;
    pointer.x = event.clientX;
    pointer.y = event.clientY;
    render();
  });
  const endDrag = (event) => {
    if (!pointer || event.pointerId !== pointer.id) return;
    if (viewport.hasPointerCapture(event.pointerId)) viewport.releasePointerCapture(event.pointerId);
    pointer = null;
    viewport.classList.remove("is-dragging");
  };
  viewport.addEventListener("pointerup", endDrag);
  viewport.addEventListener("pointercancel", endDrag);
  viewport.addEventListener("keydown", (event) => {
    if (event.key === "+" || event.key === "=") zoomCenter(1.25);
    else if (event.key === "-") zoomCenter(0.8);
    else if (event.key === "0") fit();
    else if (event.key === "ArrowLeft") panX += 60;
    else if (event.key === "ArrowRight") panX -= 60;
    else if (event.key === "ArrowUp") panY += 60;
    else if (event.key === "ArrowDown") panY -= 60;
    else return;
    event.preventDefault();
    render();
  });
  window.addEventListener("resize", () => {
    if (dialog.open) fit();
  });
  image.addEventListener("load", () => {
    if (dialog.open) fit();
  });
})();
