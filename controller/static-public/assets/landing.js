(async () => {
  const surprise = document.querySelector("[data-surprise-link]");
  if (!surprise) {
    return;
  }

  try {
    const catalog = await fetch("/data/collection.json").then((response) => response.json());
    if (!catalog.items.length) {
      const disabled = document.createElement("button");
      disabled.className = surprise.className;
      disabled.type = "button";
      disabled.disabled = true;
      disabled.textContent = surprise.textContent;
      surprise.replaceWith(disabled);
      return;
    }
    const item = catalog.items[Math.floor(Math.random() * catalog.items.length)];
    surprise.href = `/items/${encodeURIComponent(item.slug)}/`;
  } catch (_error) {
    surprise.href = "/collection/";
  }
})();
