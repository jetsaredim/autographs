const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");

const {
  cloneFramingDraftState,
  createOperationAuthority,
  createSaveCoordinator,
  framingAssessment,
} = require("./app.js");

function deferred() {
  let resolve;
  let reject;
  const promise = new Promise((done, fail) => { resolve = done; reject = fail; });
  return { promise, reject, resolve };
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

test("save coordinator suppresses duplicates and stale publication through settlement", async () => {
  let currentOwner = { imageId: 0, generation: 0, adjustmentSignature: "draft-a" };
  let currentAdjustment = { rotationDegrees: 1 };
  const persistence = [];
  const gates = [];
  const published = [];
  const pending = [];
  const errors = [];
  const coordinator = createSaveCoordinator({
    captureSnapshot: () => ({ owner: currentOwner, adjustment: structuredClone(currentAdjustment) }),
    persist(snapshot) {
      persistence.push(snapshot);
      const gate = deferred();
      gates.push(gate);
      return gate.promise;
    },
    owns: (owner) => owner === currentOwner,
    publish: (result, snapshot) => published.push({ result, snapshot }),
    setPending: (value) => pending.push(value),
    reportError: (error) => errors.push(error.message),
  });

  const first = coordinator.save();
  const duplicate = coordinator.save();
  assert.equal(persistence.length, 1, "duplicate activation dispatches no persistence call");
  assert.deepEqual(pending, [true], "pending begins synchronously");

  currentOwner = { imageId: 1, generation: 1, adjustmentSignature: "draft-b" };
  currentAdjustment = { rotationDegrees: 2 };
  gates[0].resolve({ adjustment: { rotationDegrees: 1 } });
  assert.equal(await first, false);
  assert.equal(await duplicate, false);
  assert.deepEqual(published, [], "stale completion cannot publish any UI mutation");
  assert.deepEqual(errors, []);
  assert.deepEqual(pending, [true, false], "pending clears only after settlement");

  const next = coordinator.save();
  assert.equal(persistence.length, 2, "a later explicit activation captures a new snapshot");
  assert.deepEqual(persistence[1].adjustment, { rotationDegrees: 2 });
  gates[1].resolve({ adjustment: { rotationDegrees: 2 } });
  assert.equal(await next, true);
  assert.equal(published.length, 1);
  assert.equal(published[0].snapshot.owner, currentOwner);
  assert.deepEqual(pending, [true, false, true, false]);
});

test("save coordinator reports current errors and clears pending", async () => {
  const owner = { imageId: 0, generation: 0, adjustmentSignature: "draft-a" };
  const gate = deferred();
  const pending = [];
  const errors = [];
  const coordinator = createSaveCoordinator({
    captureSnapshot: () => ({ owner, adjustment: { rotationDegrees: 0 } }),
    persist: () => gate.promise,
    owns: (candidate) => candidate === owner,
    publish: () => assert.fail("failed persistence must not publish"),
    setPending: (value) => pending.push(value),
    reportError: (error) => errors.push(error.message),
  });

  const save = coordinator.save();
  gate.reject(new Error("write failed"));
  assert.equal(await save, false);
  assert.deepEqual(errors, ["write failed"]);
  assert.deepEqual(pending, [true, false]);
});

test("production Save uses the non-aborting coordinator seam", () => {
  const source = fs.readFileSync(path.join(__dirname, "app.js"), "utf8");
  const start = source.indexOf("function saveFixture");
  const end = source.indexOf("\nfunction ", start + 1);
  const body = source.slice(start, end);
  assert.match(body, /saveCoordinator\.save/);
  assert.doesNotMatch(body, /ownedApi|AbortController/);
});

test("existing crop, zoom, or pan suppresses a crop-driving framing result", () => {
  const assessment = framingAssessment({
    margins: { top: 8, right: 8, bottom: 8, left: 8 },
    vertical: { deviation: 0 },
    horizontal: { deviation: 0 },
    maximum: 8,
    itemShortEdgePixels: 400,
    itemIntersectsInvalidPixels: false,
    limitedByExistingCrop: true,
  }, 4);

  assert.equal(assessment.status, "Constrained");
  assert.match(assessment.message, /Reset those controls/);
});

test("framing undo snapshot round-trips every manual geometry input", () => {
  const draft = {
    adjustment: {
      rotationDegrees: 1.5,
      zoom: 1,
      panX: 0,
      panY: 0,
      crop: null,
      perspective: { corners: [{ x: .1, y: .1 }, { x: .9, y: .1 }, { x: .9, y: .9 }, { x: .1, y: .9 }] },
    },
    measurementCorners: [{ x: .2, y: .2 }, { x: .8, y: .2 }, { x: .8, y: .8 }, { x: .2, y: .8 }],
    measurementTouched: new Set([0, 1, 2, 3]),
    measurementOrigin: "manual",
    manualFramingSupport: { maxSymmetricMarginPixels: 12 },
    manualFramingSignature: "confirmed-boundary",
    manualFramingReassessAfterRender: false,
  };

  const snapshot = cloneFramingDraftState(draft);
  draft.adjustment.rotationDegrees = 8;
  draft.measurementCorners[0].x = .6;
  draft.measurementTouched.delete(3);
  draft.manualFramingSupport.maxSymmetricMarginPixels = 2;

  assert.equal(snapshot.adjustment.rotationDegrees, 1.5);
  assert.equal(snapshot.measurementCorners[0].x, .2);
  assert.deepEqual(snapshot.measurementTouched, [0, 1, 2, 3]);
  assert.equal(snapshot.measurementOrigin, "manual");
  assert.deepEqual(snapshot.manualFramingSupport, { maxSymmetricMarginPixels: 12 });
  assert.equal(snapshot.manualFramingSignature, "confirmed-boundary");
  assert.equal(snapshot.manualFramingReassessAfterRender, false);
});
