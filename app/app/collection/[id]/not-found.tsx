import { Breadcrumbs } from "../../components/Breadcrumbs";
import { EmptyState } from "../../components/EmptyState";

export default function CollectionItemNotFound() {
  return (
    <main className="site-shell">
      <section className="detail-heading">
        <Breadcrumbs
          items={[
            { label: "Home", href: "/" },
            { label: "Collection", href: "/collection" },
            { label: "Missing autograph" },
          ]}
        />
      </section>
      <EmptyState variant="not-found" />
    </main>
  );
}
