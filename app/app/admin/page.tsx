import Link from "next/link";

export default function AdminPlaceholderPage() {
  return (
    <main className="architecture-shell">
      <section className="admin-placeholder" aria-labelledby="admin-placeholder-title">
        <nav className="breadcrumbs" aria-label="Breadcrumb">
          <Link href="/">Home</Link>
          <span aria-hidden="true">&gt;</span>
          <span>Collection management</span>
        </nav>
        <h1 id="admin-placeholder-title">Collection management is coming later</h1>
        <p className="lede">
          Phase 4 will add the real single-owner workflow for creating,
          editing, curating, and maintaining collection records. This page is
          only a placeholder so the access pattern can settle before that work
          begins.
        </p>
      </section>
    </main>
  );
}
