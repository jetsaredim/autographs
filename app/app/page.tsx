const healthPayload = `{"ok":true,"service":"autographs","scope":"proof-of-life"}`;

export default function HomePage() {
  return (
    <main className="proof-shell">
      <section className="proof-card" aria-labelledby="proof-title">
        <p className="eyebrow">Phase 1 proof-of-life</p>
        <h1 id="proof-title">Autographs is up and ready for the delivery spine.</h1>
        <p className="lede">
          This landing page is intentionally narrow. It proves the Next.js app
          scaffold exists, renders through the App Router, and exposes a stable
          health surface before gallery, admin, Oracle, or object-storage work
          begins.
        </p>

        <div className="status-grid">
          <div className="status-tile">
            <strong>App router</strong>
            <span>Layout and page entrypoints are in place under `app/app`.</span>
          </div>
          <div className="status-tile">
            <strong>Health route</strong>
            <span>
              Machine checks can call <a href="/health">/health</a> for a stable
              JSON success response.
            </span>
          </div>
          <div className="status-tile">
            <strong>Response contract</strong>
            <code>{healthPayload}</code>
          </div>
        </div>
      </section>
    </main>
  );
}
