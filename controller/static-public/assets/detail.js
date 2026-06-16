(() => {
  const viewer = document.querySelector(".image-viewer");
  const focusedButton = viewer?.querySelector(".focused-image-button");
  const focusedImage = focusedButton?.querySelector("img");
  const metadataPanel = viewer?.querySelector(".detail-metadata-panel");

  if (!viewer || !metadataPanel) {
    return;
  }

  if (!focusedButton || !focusedImage) {
    metadataPanel.classList.add("is-revealed");
    metadataPanel.setAttribute("aria-hidden", "false");
    return;
  }

  const setRevealed = (revealed) => {
    viewer.classList.toggle("is-revealed", revealed);
    metadataPanel.classList.toggle("is-revealed", revealed);
    focusedButton.setAttribute("aria-expanded", String(revealed));
    metadataPanel.setAttribute("aria-hidden", String(!revealed));
  };

  focusedButton.addEventListener("click", () => setRevealed(!viewer.classList.contains("is-revealed")));
  focusedButton.addEventListener("contextmenu", (event) => event.preventDefault());

  viewer.querySelectorAll(".thumbnail-button").forEach((button) => {
    button.addEventListener("click", () => {
      focusedImage.src = button.dataset.detailSrc || focusedImage.src;
      focusedImage.alt = button.dataset.detailAlt || focusedImage.alt;
      if (button.dataset.detailWidth) {
        focusedImage.width = Number(button.dataset.detailWidth);
      }
      if (button.dataset.detailHeight) {
        focusedImage.height = Number(button.dataset.detailHeight);
      }
      viewer
        .querySelectorAll(".thumbnail-button")
        .forEach((candidate) => candidate.setAttribute("aria-pressed", "false"));
      button.setAttribute("aria-pressed", "true");
    });
    button.addEventListener("contextmenu", (event) => event.preventDefault());
  });
})();
