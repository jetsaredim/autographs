const endpoints = {
  health: "/admin/api/health",
  login: "/admin/api/login",
  logout: "/admin/api/logout",
  items: "/admin/api/items",
  publishIncremental: "/admin/api/publish/incremental",
  publishFull: "/admin/api/publish/full",
  publishStatus: "/admin/api/publish/status",
};

const sessionMessage = document.querySelector("#session-message");
const itemMessage = document.querySelector("#item-message");
const publishStatus = document.querySelector("#publish-status");
const itemForm = document.querySelector("#item-form");

const request = async (path, options = {}) => {
  const response = await fetch(path, {
    credentials: "same-origin",
    ...options,
  });
  if (!response.ok) {
    const detail = await response.text();
    throw new Error(detail || `${response.status} ${response.statusText}`);
  }
  if (response.status === 204) {
    return null;
  }
  return response.json();
};

const jsonRequest = (path, method, body) =>
  request(path, {
    method,
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });

const optionalValue = (form, name) => {
  const value = form.elements[name].value.trim();
  return value || null;
};

const itemId = () => itemForm.elements.itemId.value.trim();

const renderStatus = async () => {
  try {
    const [health, status] = await Promise.all([
      request(endpoints.health),
      request(endpoints.publishStatus),
    ]);
    publishStatus.textContent = JSON.stringify({ health, publish: status }, null, 2);
  } catch (error) {
    publishStatus.textContent = `Unable to load publish status: ${error.message}`;
  }
};

document.querySelector("#login-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  sessionMessage.textContent = "";
  try {
    await jsonRequest(endpoints.login, "POST", {
      password: event.currentTarget.elements.password.value,
    });
    event.currentTarget.reset();
    sessionMessage.textContent = "Logged in.";
    await renderStatus();
  } catch (error) {
    sessionMessage.textContent = `Login failed: ${error.message}`;
  }
});

document.querySelector("#logout").addEventListener("click", async () => {
  try {
    await request(endpoints.logout, { method: "POST" });
    sessionMessage.textContent = "Logged out.";
    publishStatus.textContent = "Not loaded.";
  } catch (error) {
    sessionMessage.textContent = `Logout failed: ${error.message}`;
  }
});

const triggerPublish = async (path) => {
  publishStatus.textContent = "Publishing...";
  try {
    const status = await request(path, { method: "POST" });
    publishStatus.textContent = JSON.stringify(status, null, 2);
  } catch (error) {
    publishStatus.textContent = `Publish failed: ${error.message}`;
  }
};

document.querySelector("#refresh-status").addEventListener("click", renderStatus);
document
  .querySelector("#publish-incremental")
  .addEventListener("click", () => triggerPublish(endpoints.publishIncremental));
document
  .querySelector("#publish-full")
  .addEventListener("click", () => triggerPublish(endpoints.publishFull));

const setPublication = async (publicationStatus) => {
  const id = itemId();
  if (!id) {
    throw new Error("Save or enter an item ID first.");
  }
  return jsonRequest(`${endpoints.items}/${id}/publication`, "POST", {
    publicationStatus,
  });
};

document.querySelector("#publish-item").addEventListener("click", async () => {
  try {
    await setPublication("published");
    itemMessage.textContent = "Item marked published. Trigger a static publish to update the public release.";
  } catch (error) {
    itemMessage.textContent = `Publish item failed: ${error.message}`;
  }
});

document.querySelector("#unpublish-item").addEventListener("click", async () => {
  try {
    await setPublication("draft");
    itemMessage.textContent = "Item returned to draft. Trigger a static publish to remove public artifacts.";
  } catch (error) {
    itemMessage.textContent = `Unpublish item failed: ${error.message}`;
  }
});

itemForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  itemMessage.textContent = "";
  const form = event.currentTarget;
  const id = itemId();
  const estimatedYear = optionalValue(form, "estimatedYear");
  const payload = {
    title: form.elements.title.value.trim(),
    signer: form.elements.signer.value.trim(),
    description: optionalValue(form, "description"),
    category: form.elements.category.value.trim(),
    tags: form.elements.tags.value.split(",").map((tag) => tag.trim()).filter(Boolean),
    objectReference: optionalValue(form, "objectReference"),
    eventName: optionalValue(form, "eventName"),
    eventLocation: optionalValue(form, "eventLocation"),
    source: optionalValue(form, "source"),
    inscription: optionalValue(form, "inscription"),
    certificationCompany: optionalValue(form, "certificationCompany"),
    certificationId: optionalValue(form, "certificationId"),
    estimatedYear: estimatedYear ? Number(estimatedYear) : null,
    publicationStatus: form.elements.publicationStatus.value,
  };

  try {
    const item = await jsonRequest(id ? `${endpoints.items}/${id}` : endpoints.items, id ? "PATCH" : "POST", payload);
    form.elements.itemId.value = item.id;
    const image = form.elements.image.files[0];
    if (image) {
      const upload = new FormData();
      upload.append("image", image);
      upload.append("altText", optionalValue(form, "altText") || "");
      await request(`${endpoints.items}/${item.id}/images`, {
        method: "POST",
        body: upload,
      });
    }
    itemMessage.textContent = `Saved ${item.id}.`;
  } catch (error) {
    itemMessage.textContent = `Save failed: ${error.message}`;
  }
});

renderStatus();
