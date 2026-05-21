import assert from "node:assert/strict";
import test from "node:test";

import { approvedQuotes, selectApprovedQuote } from "./approved-quotes";

test("approved quotes contains the user-approved inventory", () => {
  assert.equal(approvedQuotes.length, 20);

  const ids = new Set(approvedQuotes.map((quote) => quote.id));
  assert.equal(ids.size, approvedQuotes.length);
});

test("approved quotes have quote text, attribution, and short public copy", () => {
  for (const approvedQuote of approvedQuotes) {
    assert.ok(approvedQuote.quote.trim(), `${approvedQuote.id} needs quote text`);
    assert.ok(approvedQuote.attribution.trim(), `${approvedQuote.id} needs attribution`);
    assert.ok(approvedQuote.tone.trim(), `${approvedQuote.id} needs tone`);
    assert.ok(
      approvedQuote.quote.split(/\s+/).length < 25,
      `${approvedQuote.id} must stay under 25 words`,
    );
  }
});

test("selectApprovedQuote returns a quote from the approved inventory", () => {
  assert.equal(selectApprovedQuote(0), approvedQuotes[0]);
  assert.equal(selectApprovedQuote(approvedQuotes.length + 2), approvedQuotes[2]);
  assert.equal(selectApprovedQuote(-1), approvedQuotes[1]);
});
