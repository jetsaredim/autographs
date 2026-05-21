import type { AutographImage, AutographItem } from "./types";

export type PublicImage = {
  id: string;
  src: string;
  altText: string;
};

export type PublicGalleryItem = {
  id: string;
  title: string;
  signer: string;
  description: string | null;
  category: string;
  tags: string[];
  primaryImage: PublicImage | null;
};

export type PublicDetailField = {
  label: string;
  value: string;
};

export type PublicDetailGroup = {
  label: string;
  fields: PublicDetailField[];
};

export type PublicItemDetail = PublicGalleryItem & {
  images: PublicImage[];
  detailGroups: PublicDetailGroup[];
};

export type PublicFacetOption = {
  label: string;
  value: string;
};

export type PublicFacetGroup = {
  id: "signer" | "category" | "tag";
  label: string;
  options: PublicFacetOption[];
};

export const buildPublicImageSrc = (itemId: string, imageId: string): string =>
  `/api/catalog/${encodeURIComponent(itemId)}/images/${encodeURIComponent(imageId)}`;

export const toPublicGalleryItem = (item: AutographItem): PublicGalleryItem => ({
  id: item.id,
  title: item.title,
  signer: item.signer,
  description: item.description ?? null,
  category: item.category,
  tags: item.tags.slice(0, 3),
  primaryImage: toPublicImage(item, selectPrimaryImage(item.images)),
});

export const toPublicItemDetail = (item: AutographItem): PublicItemDetail => ({
  ...toPublicGalleryItem(item),
  images: sortImages(item.images)
    .map((image) => toPublicImage(item, image))
    .filter(isPresent),
  detailGroups: buildDetailGroups(item),
});

export const derivePublicFacets = (items: AutographItem[]): PublicFacetGroup[] => [
  {
    id: "signer",
    label: "Signer",
    options: toOptions(items.map((item) => item.signer)),
  },
  {
    id: "category",
    label: "Category",
    options: toOptions(items.map((item) => item.category)),
  },
  {
    id: "tag",
    label: "Tags",
    options: toOptions(items.flatMap((item) => item.tags)),
  },
];

const toPublicImage = (item: AutographItem, image: AutographImage | null): PublicImage | null => {
  if (!image) {
    return null;
  }

  return {
    id: image.id,
    src: buildPublicImageSrc(item.id, image.id),
    altText: image.altText ?? `${item.title} signed by ${item.signer}`,
  };
};

const selectPrimaryImage = (images: AutographImage[]): AutographImage | null => {
  const [first] = sortImages(images);
  return images.find((image) => image.isPrimary) ?? first ?? null;
};

const sortImages = (images: AutographImage[]): AutographImage[] =>
  [...images].sort((left, right) => left.sortOrder - right.sortOrder);

const toOptions = (values: Array<string | null | undefined>): PublicFacetOption[] =>
  [...new Set(values.map((value) => value?.trim()).filter(isPresent))]
    .sort((left, right) => left.localeCompare(right))
    .map((value) => ({ label: value, value }));

const buildDetailGroups = (item: AutographItem): PublicDetailGroup[] =>
  [
    group("Essentials", [
      field("Signer", item.signer),
      field("Title", item.title),
      field("Category", item.category),
      field("Estimated year", item.estimatedYear?.toString()),
    ]),
    group("Provenance", [
      field("Event", joinParts([item.eventName, item.eventLocation])),
      field("Source", item.source),
    ]),
    group("Certification", [
      field("Company", item.certificationCompany),
      field("Identifier", item.certificationId),
    ]),
    group("Inscription", [field("Text", item.inscription)]),
    group("Tags", item.tags.map((tag) => field("Tag", tag))),
    group("Collection Notes", [
      field("Description", item.description),
      field("Object reference", item.objectReference),
    ]),
  ].filter(isPresent);

const group = (
  label: string,
  fields: Array<PublicDetailField | null>,
): PublicDetailGroup | null => {
  const visibleFields = fields.filter(isPresent);
  if (visibleFields.length === 0) {
    return null;
  }

  return {
    label,
    fields: visibleFields,
  };
};

const field = (label: string, value: string | null | undefined): PublicDetailField | null => {
  const normalized = value?.trim();
  if (!normalized) {
    return null;
  }

  return {
    label,
    value: normalized,
  };
};

const joinParts = (parts: Array<string | null | undefined>): string | null => {
  const visibleParts = parts.map((part) => part?.trim()).filter(isPresent);
  return visibleParts.length > 0 ? visibleParts.join(", ") : null;
};

const isPresent = <T>(value: T | null | undefined | ""): value is T =>
  value !== null && value !== undefined && value !== "";
