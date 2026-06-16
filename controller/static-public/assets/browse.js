(async () => {
  const root = document.querySelector("#collection");
  const count = document.querySelector("#collection-count");
  const panel = document.querySelector("#collection-filters");
  const menu = document.querySelector(".filter-menu");
  const chips = document.querySelector(".selected-filters");
  const toggle = document.querySelector(".filter-toggle");

  if (!root || !count || !panel || !menu || !chips || !toggle) {
    return;
  }

  const [catalog, facets] = await Promise.all([
    fetch("/data/collection.json").then((response) => response.json()),
    fetch("/data/facets.json").then((response) => response.json()),
  ]);
  const params = new URLSearchParams(window.location.search);
  const state = {
    signer: normalizedFilter(params.get("signer")),
    category: normalizedFilter(params.get("category")),
    tag: normalizedFilter(params.get("tag")),
  };
  const selects = new Map();

  const setToggleIcon = (open) => {
    toggle.replaceChildren(
      icon(open ? "M6 6l12 12M18 6L6 18" : "M4 6h16l-6.5 7.5V19l-3 1.5v-7z"),
    );
  };

  const facet = (id) => facets.groups.find((group) => group.id === id) || { id, label: id, options: [] };
  const option = (value, label) => {
    const node = document.createElement("option");
    node.value = value;
    node.textContent = label;
    return node;
  };
  const select = (group) => {
    const node = document.createElement("select");
    node.setAttribute("aria-label", group.label);
    node.replaceChildren(
      option("all", group.label),
      ...group.options.map((item) => option(item.value, item.label)),
    );
    node.value = state[group.id] || "all";
    node.addEventListener("change", () => updateFilter(group.id, node.value));
    selects.set(group.id, node);
    return node;
  };
  const syncUrl = () => {
    const next = new URLSearchParams();
    Object.entries(state).forEach(([key, value]) => {
      if (value) {
        next.set(key, value);
      }
    });
    const query = next.toString();
    const url = `/collection/${query ? `?${query}` : ""}`;
    window.history.pushState({ ...state }, "", url);
  };
  const updateFilter = (id, value) => {
    state[id] = normalizedFilter(value);
    syncUrl();
    render();
  };
  const setOpen = (open) => {
    panel.hidden = !open;
    toggle.setAttribute("aria-expanded", String(open));
    toggle.setAttribute("aria-label", open ? "Close filters" : "Open filters");
    setToggleIcon(open);
  };

  menu.replaceChildren(select(facet("signer")), select(facet("category")), select(facet("tag")));
  setOpen(Object.values(state).some(Boolean));
  toggle.addEventListener("click", () => setOpen(panel.hidden));
  window.addEventListener("popstate", () => {
    const next = new URLSearchParams(window.location.search);
    state.signer = normalizedFilter(next.get("signer"));
    state.category = normalizedFilter(next.get("category"));
    state.tag = normalizedFilter(next.get("tag"));
    for (const [id, node] of selects) {
      node.value = state[id] || "all";
    }
    setOpen(Object.values(state).some(Boolean));
    render();
  });

  function render() {
    const filtered = catalog.items.filter(
      (item) =>
        (!state.signer || item.signer === state.signer) &&
        (!state.category || item.category === state.category) &&
        (!state.tag || item.tags.includes(state.tag)),
    );
    count.textContent =
      filtered.length === 1 ? "1 published autograph" : `${filtered.length} published autographs`;
    chips.replaceChildren(
      ...Object.entries(state)
        .filter(([, value]) => value)
        .map(([id, value]) => filterChip(facet(id), value)),
    );
    root.replaceChildren(...(filtered.length > 0 ? filtered.map(card) : [emptyState()]));
  }

  const filterChip = (group, value) => {
    const label = (group.options.find((item) => item.value === value) || { label: value }).label;
    const chip = document.createElement("button");
    chip.className = "filter-chip";
    chip.type = "button";
    chip.textContent = `${group.label}: ${label}`;
    chip.addEventListener("click", () => {
      const selectNode = selects.get(group.id);
      if (selectNode) {
        selectNode.value = "all";
      }
      updateFilter(group.id, "all");
    });
    return chip;
  };

  const card = (item) => {
    const link = Object.assign(document.createElement("a"), {
      className: "gallery-card-link",
      href: `/collection/${encodeURIComponent(item.slug)}/`,
    });
    link.setAttribute("aria-label", `${item.title} signed by ${item.signer}`);
    const article = Object.assign(document.createElement("article"), { className: "gallery-card" });
    const media = Object.assign(document.createElement("div"), { className: "gallery-card-media" });
    media.addEventListener("contextmenu", (event) => event.preventDefault());
    const image = variant(item, "thumbnail");
    if (image) {
      const img = Object.assign(document.createElement("img"), {
        src: image.path,
        alt: item.primaryImage.altText,
        width: image.width,
        height: image.height,
        draggable: false,
      });
      media.append(img);
    } else {
      media.append(text(document.createElement("span"), "No image published yet"));
    }
    const overlay = Object.assign(document.createElement("div"), { className: "gallery-card-overlay" });
    overlay.append(text(document.createElement("span"), item.signer));
    media.append(overlay);
    article.append(media);
    link.append(article);
    return link;
  };

  render();
})();

const normalizedFilter = (value) => (value && value !== "all" ? value : "");
const text = (node, value) => {
  node.textContent = value;
  return node;
};
const variant = (item, name) =>
  item.primaryImage?.variants?.find((entry) => entry.name === name) || item.primaryImage?.variants?.[0];
const emptyState = () => {
  const section = Object.assign(document.createElement("section"), { className: "empty-state" });
  section.dataset.emptyState = "no-results";
  const copy = Object.assign(document.createElement("div"), { className: "empty-state-copy" });
  const title = text(document.createElement("h2"), "No autographs match those filters.");
  const body = text(document.createElement("p"), "Clear a filter or return to the full collection.");
  const link = Object.assign(document.createElement("a"), {
    className: "button-secondary",
    href: "/collection/",
    textContent: "View collection",
  });
  copy.append(title, body, link);
  section.append(copy);
  return section;
};
const icon = (pathData) => {
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
  svg.setAttribute("aria-hidden", "true");
  svg.setAttribute("viewBox", "0 0 24 24");
  path.setAttribute("d", pathData);
  svg.append(path);
  return svg;
};
