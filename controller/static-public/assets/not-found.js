(async () => {
  const quoteNode = document.querySelector("[data-not-found-quote]");
  const sourceNode = document.querySelector("[data-not-found-source]");

  if (!quoteNode || !sourceNode) {
    return;
  }

  try {
    const quotes = await fetch("/data/not-found-quotes.json").then((response) => response.json());
    const approved = quotes.filter((entry) => entry.quote && entry.source);

    if (!approved.length) {
      return;
    }

    const selected = approved[Math.floor(Math.random() * approved.length)];
    quoteNode.textContent = `“${selected.quote}”`;
    sourceNode.textContent = selected.source;
  } catch (_error) {
    quoteNode.textContent = "Page not found.";
    sourceNode.textContent = "";
  }
})();
