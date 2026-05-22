import { Breadcrumbs } from "../components/Breadcrumbs";
import { PublicFooter } from "../components/PublicFooter";

export default function AdminPlaceholderPage() {
  return (
    <main className="architecture-shell">
      <section className="admin-placeholder" aria-labelledby="admin-placeholder-title">
        <Breadcrumbs items={[{ label: "Home", href: "/" }, { label: "Collection management" }]} />
        <h1 id="admin-placeholder-title">Collection management is coming later</h1>
        <p className="lede">
          Phase 4 will add the real single-owner workflow for creating,
          editing, curating, and maintaining collection records. This page is
          only a placeholder so the access pattern can settle before that work
          begins.
        </p>
      </section>

      <PublicFooter />
    </main>
  );
}
