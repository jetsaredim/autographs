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

  const controlFilterIds = ["signer", "franchise", "productLine", "format", "language", "origin", "role"];
  const queryFilterIds = [...controlFilterIds, "tag"];
  const languageDisplay = new Map([
    ["", { label: "🌐 All", title: "All languages" }],
    ["English", { label: "🇺🇸 EN", title: "English" }],
    ["Japanese", { label: "🇯🇵 JA", title: "Japanese" }],
    ["Chinese", { label: "🇨🇳 ZH", title: "Chinese" }],
  ]);
  const params = new URLSearchParams(window.location.search);
  const state = Object.fromEntries(queryFilterIds.map((id) => [id, normalizedFilter(params.get(id))]));
  const selects = new Map();
  let catalog;
  let facets;

  try {
    [catalog, facets] = await Promise.all([
      fetch("/data/collection.json").then((response) => {
        if (!response.ok) {
          throw new Error("collection failed");
        }
        return response.json();
      }),
      fetch("/data/facets.json").then((response) => {
        if (!response.ok) {
          throw new Error("facets failed");
        }
        return response.json();
      }),
    ]);
  } catch {
    count.textContent = "Collection unavailable";
    root.replaceChildren(facetLoadError());
    return;
  }

  const activeFilterCount = () => Object.values(state).filter(Boolean).length;

  const syncToggleHint = () => {
    const count = activeFilterCount();

    toggle.classList.toggle("has-active-filters", count > 0);

    if (count > 0) {
      toggle.dataset.activeFilterCount = String(count);
    } else {
      delete toggle.dataset.activeFilterCount;
    }
  };

  const setToggleIcon = (open) => {
    toggle.replaceChildren(
      icon(open ? "M6 6l12 12M18 6L6 18" : "M4 6h16l-6.5 7.5V19l-3 1.5v-7z"),
    );
  };

  const facet = (id) => facets.groups.find((group) => group.id === id) || { id, label: id, options: [] };
  const languageOption = (value, label) => languageDisplay.get(value) || { label, title: label };
  const option = (value, label, title = label) => {
    const node = document.createElement("option");
    node.value = value;
    node.textContent = label;
    if (title) {
      node.setAttribute("aria-label", title);
      node.title = title;
    }
    return node;
  };
  const select = (group) => {
    const node = document.createElement("select");
    const isLanguage = group.id === "language";
    const allLanguages = languageOption("", group.label);
    node.setAttribute("aria-label", isLanguage ? "Item language" : group.label);
    if (isLanguage) {
      node.title = "Item language";
    }
    node.replaceChildren(
      isLanguage ? option("all", allLanguages.label, allLanguages.title) : option("all", group.label),
      ...group.options.map((item) => {
        const display = isLanguage
          ? languageOption(item.value, item.label)
          : { label: item.label, title: item.label };
        return option(item.value, display.label, display.title);
      }),
    );
    node.value = state[group.id] || "all";
    node.addEventListener("change", () => updateFilter(group.id, node.value));
    selects.set(group.id, node);
    return node;
  };
  const filterControls = (ids) => {
    const controls = Object.assign(document.createElement("div"), { className: "filter-controls" });
    controls.replaceChildren(...ids.map((id) => select(facet(id))));
    return controls;
  };
  const syncUrl = () => {
    const next = new URLSearchParams();
    queryFilterIds.forEach((key) => {
      if (state[key]) {
        next.set(key, state[key]);
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
  const clearFilters = () => {
    queryFilterIds.forEach((id) => {
      state[id] = "";
      const node = selects.get(id);
      if (node) {
        node.value = "all";
      }
    });
    syncUrl();
    render();
  };
  const setOpen = (open) => {
    panel.classList.toggle("is-collapsed", !open);
    panel.setAttribute("aria-hidden", String(!open));
    panel.inert = !open;

    toggle.setAttribute("aria-expanded", String(open));
    toggle.setAttribute("aria-label", open ? "Close filters" : "Open filters");
    setToggleIcon(open);
    syncToggleHint();
  };

  menu.replaceChildren(filterControls(controlFilterIds));

  setOpen(Object.values(state).some(Boolean));

  toggle.addEventListener("click", () => {
    setOpen(panel.classList.contains("is-collapsed"));
  });
  window.addEventListener("popstate", () => {
    const next = new URLSearchParams(window.location.search);
    queryFilterIds.forEach((id) => {
      state[id] = normalizedFilter(next.get(id));
      const node = selects.get(id);
      if (node) {
        node.value = state[id] || "all";
      }
    });
    setOpen(Object.values(state).some(Boolean));
    render();
  });

  function render() {
    const filtered = catalog.items.filter(matchesFilters);
    count.textContent =
      filtered.length === 1 ? "1 published autograph" : `${filtered.length} published autographs`;
    chips.replaceChildren(
      ...queryFilterIds
        .filter((id) => state[id])
        .map((id) => filterChip(facet(id), state[id])),
    );
    syncToggleHint();
    root.replaceChildren(...(filtered.length > 0 ? filtered.map(card) : [emptyState(clearFilters)]));
  }

  const matchesFilters = (item) =>
    (!state.signer || values(item.signerNames).includes(state.signer)) &&
    (!state.franchise || values(item.franchises).includes(state.franchise)) &&
    (!state.productLine || item.productLine === state.productLine) &&
    (!state.format || item.format === state.format) &&
    (!state.language || item.language === state.language) &&
    (!state.origin || item.origin === state.origin) &&
    (!state.role || values(item.signerRoles).includes(state.role)) &&
    (!state.tag || values(item.tags).includes(state.tag));

  const filterChip = (group, value) => {
    const label = (group.options.find((item) => item.value === value) || { label: value }).label;
    const chip = document.createElement("button");
    chip.className = "filter-chip";
    chip.type = "button";
    chip.textContent = `${group.label}: ${label}`;
    chip.title = `${group.label}: ${label}`;
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
    const signerText = item.signerText || values(item.signerNames).join(", ");
    const signerNames = values(item.signerNames);
    const link = Object.assign(document.createElement("a"), {
      className: "gallery-card-link",
      href: `/items/${encodeURIComponent(item.slug)}/`,
    });
    link.setAttribute("aria-label", `${item.title} signed by ${signerNames.join(", ") || signerText}`);
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
    overlay.append(text(document.createElement("span"), signerText));
    media.append(overlay);
    article.append(media);
    link.append(article);
    return link;
  };

  render();
})();

function normalizedFilter(value) {
  return value && value !== "all" ? value : "";
}

function values(value) {
  return Array.isArray(value) ? value : [];
}

function text(node, value) {
  node.textContent = value;
  return node;
}

function variant(item, name) {
  return item.primaryImage?.variants?.find((entry) => entry.name === name) || item.primaryImage?.variants?.[0];
}

function emptyState(clearFilters) {
  const section = Object.assign(document.createElement("section"), { className: "empty-state" });
  section.dataset.emptyState = "no-results";
  const copy = Object.assign(document.createElement("div"), { className: "empty-state-copy" });
  const title = text(document.createElement("h2"), "No autographs match those filters.");
  const body = text(document.createElement("p"), "Clear filters or return to the full collection.");
  const actions = Object.assign(document.createElement("div"), { className: "empty-state-actions" });
  const clear = Object.assign(document.createElement("button"), {
    className: "button-secondary",
    type: "button",
    textContent: "Clear filters",
  });
  clear.addEventListener("click", clearFilters);
  const link = Object.assign(document.createElement("a"), {
    className: "button-secondary",
    href: "/collection/",
    textContent: "View collection",
  });
  actions.append(clear, link);
  copy.append(title, body, actions);
  section.append(copy);
  return section;
}

function facetLoadError() {
  const section = Object.assign(document.createElement("section"), { className: "empty-state" });
  section.dataset.emptyState = "facet-error";
  const copy = Object.assign(document.createElement("div"), { className: "empty-state-copy" });
  const message = text(
    document.createElement("p"),
    "The collection facets could not be loaded. Refresh the page or return to the full collection.",
  );
  const actions = Object.assign(document.createElement("div"), { className: "empty-state-actions" });
  const refresh = Object.assign(document.createElement("button"), {
    className: "button-secondary",
    type: "button",
    textContent: "Refresh",
  });
  refresh.addEventListener("click", () => window.location.reload());
  const link = Object.assign(document.createElement("a"), {
    className: "button-secondary",
    href: "/collection/",
    textContent: "View collection",
  });
  actions.append(refresh, link);
  copy.append(message, actions);
  section.append(copy);
  return section;
}

function icon(pathData) {
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
  svg.setAttribute("aria-hidden", "true");
  svg.setAttribute("viewBox", "0 0 24 24");
  path.setAttribute("d", pathData);
  svg.append(path);
  return svg;
}
