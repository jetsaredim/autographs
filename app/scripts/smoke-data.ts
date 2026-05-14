import { createCatalogRepository } from "../src/catalog";
import { createPrivateMediaStore } from "../src/media";

const main = async (): Promise<void> => {
  await createCatalogRepository().list({ includeUnpublished: true });
  await createPrivateMediaStore().assertReady();
  console.log(JSON.stringify({ ok: true, scope: "data-live" }));
};

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
