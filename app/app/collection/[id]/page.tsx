import { notFound } from "next/navigation";

import { ImageViewer } from "../../components/ImageViewer";
import { getPublishedCatalogItem } from "../../catalog-data";
import { toPublicItemDetail } from "../../../src/catalog/public-view-models";

export const dynamic = "force-dynamic";

type DetailPageProps = {
  params: Promise<{ id: string }>;
};

const detailSectionOrder = [
  "Provenance",
  "Certification",
  "Inscription",
  "Tags",
  "Collection Notes",
];

export default async function CollectionDetailPage({ params }: DetailPageProps) {
  const { id } = await params;
  const item = await getPublishedCatalogItem(id);
  if (!item) {
    notFound();
  }

  const detail = toPublicItemDetail(item);
  const certification = findField(detail.detailGroups, "Certification", "Company");
  const metadataGroups = detailSectionOrder
    .map((label) => detail.detailGroups.find((group) => group.label === label))
    .filter((group) => group !== undefined);

  const metadataPanel = (
    <div className="detail-metadata">
      <div className="detail-facts">
        <span>{detail.signer}</span>
        <span>{detail.category}</span>
        {findField(detail.detailGroups, "Essentials", "Estimated year") ? (
          <span>{findField(detail.detailGroups, "Essentials", "Estimated year")}</span>
        ) : null}
        {certification ? <span>{certification}</span> : null}
      </div>

      {metadataGroups.map((group) => (
        <section className="metadata-group" key={group.label}>
          <h2>{group.label}</h2>
          <dl>
            {group.fields.map((field, index) => (
              <div key={`${field.label}-${field.value}-${index}`}>
                <dt>{field.label}</dt>
                <dd>{field.value}</dd>
              </div>
            ))}
          </dl>
        </section>
      ))}
    </div>
  );

  return (
    <main className="site-shell detail-shell">
      <header className="detail-heading">
        <p className="eyebrow">{detail.category}</p>
        <h1>{detail.title}</h1>
        <p className="lede">Signed by {detail.signer}</p>
      </header>

      <ImageViewer
        images={detail.images}
        title={detail.title}
        signer={detail.signer}
        metadataPanel={metadataPanel}
      />
    </main>
  );
}

const findField = (
  groups: Array<{ label: string; fields: Array<{ label: string; value: string }> }>,
  groupLabel: string,
  fieldLabel: string,
): string | null =>
  groups
    .find((group) => group.label === groupLabel)
    ?.fields.find((field) => field.label === fieldLabel)?.value ?? null;
