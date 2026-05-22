import { Breadcrumbs } from "./components/Breadcrumbs";
import { EmptyState } from "./components/EmptyState";

export default function NotFound() {
  return (
    <main className="site-shell">
      <section className="detail-heading">
        <Breadcrumbs items={[{ label: "Home", href: "/" }, { label: "Missing page" }]} />
      </section>
      <EmptyState variant="not-found" />
    </main>
  );
}
