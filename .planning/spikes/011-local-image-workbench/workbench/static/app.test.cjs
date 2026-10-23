const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");

const { createOperationAuthority } = require("./app.js");

function deferred() {
  let resolve;
  const promise = new Promise((done) => { resolve = done; });
  return { promise, resolve };
}

for (const operation of ["load", "detect", "save", "render", "auto-align"]) {
  test(`${operation} completion cannot publish after selection changes`, async () => {
    const state = { imageId: 0, adjustmentSignature: "draft-a" };
    const authority = createOperationAuthority(() => state);
    const owner = authority.capture({ matchAdjustment: operation !== "auto-align" });
    const completion = deferred();
    let published = null;
    const pending = completion.promise.then((value) => {
      if (authority.owns(owner)) published = value;
    });

    authority.invalidate();
    state.imageId = 1;
    state.adjustmentSignature = "draft-b";
    completion.resolve(`${operation}-result`);
    await pending;

    assert.equal(published, null);
  });
}

test("draft revisions invalidate signature-owned operations synchronously", async () => {
  const state = { imageId: 0, adjustmentSignature: "draft-a" };
  const authority = createOperationAuthority(() => state);
  const owner = authority.capture();
  const completion = deferred();
  let published = false;
  const pending = completion.promise.then(() => {
    if (authority.owns(owner)) published = true;
  });

  state.adjustmentSignature = "draft-b";
  authority.invalidate();
  completion.resolve();
  await pending;

  assert.equal(published, false);
});

test("all async mutation paths use the shared ownership contract", () => {
  const source = fs.readFileSync(path.join(__dirname, "app.js"), "utf8");
  for (const functionName of [
    "selectImage",
    "reloadFixture",
    "renderPreview",
    "detect",
    "saveFixture",
    "assessManualFraming",
    "applySuggestedAlignment",
  ]) {
    const start = source.indexOf(`function ${functionName}`);
    assert.notEqual(start, -1, `${functionName} exists`);
    const nextFunction = source.indexOf("\nfunction ", start + 1);
    const nextAsyncFunction = source.indexOf("\nasync function ", start + 1);
    const ends = [nextFunction, nextAsyncFunction].filter((index) => index !== -1);
    const end = ends.length ? Math.min(...ends) : source.length;
    const body = source.slice(start, end);
    assert.match(body, /operationAuthority\.capture/, `${functionName} captures ownership`);
    assert.match(body, /operationAuthority\.owns|ownedApi/, `${functionName} checks ownership`);
  }
});
