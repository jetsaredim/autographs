(() => {
  const unlockSequence = "gallery";
  let progress = "";
  let revealed = false;

  const revealAdminLink = () => {
    if (revealed) {
      return;
    }
    const footer = document.querySelector(".public-footer");
    if (!footer) {
      return;
    }

    revealed = true;

    const separator = document.createElement("span");
    separator.setAttribute("aria-hidden", "true");
    separator.textContent = "•";

    const link = document.createElement("a");
    link.className = "admin-unlock";
    link.href = "/admin";
    link.textContent = "Admin";

    footer.append(separator, link);
  };

  window.addEventListener("keydown", (event) => {
    if (event.altKey || event.ctrlKey || event.metaKey) {
      return;
    }

    const key = event.key.toLowerCase();
    if (key.length !== 1) {
      return;
    }

    const next = `${progress}${key}`;
    const suffix = unlockSequence.slice(0, next.length);
    const normalized = unlockSequence.startsWith(next) ? next : key;

    if (normalized === unlockSequence) {
      progress = "";
      revealAdminLink();
      return;
    }

    progress = suffix === normalized ? normalized : "";
  });
})();
