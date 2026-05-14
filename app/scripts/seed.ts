import { sampleAutographs } from "../db/seed/sample-autographs";
import { OracleCatalogRepository } from "../src/catalog/repository";
import { createOracleExecutor } from "../src/db/oracle";

const dryRun = process.argv.includes("--dry-run");

if (dryRun) {
  console.log(JSON.stringify({ dryRun: true, records: sampleAutographs }, null, 2));
  process.exit(0);
}

const repository = new OracleCatalogRepository(createOracleExecutor());

for (const item of sampleAutographs) {
  const created = await repository.create(item);
  console.log(`seeded ${created.id} ${created.title}`);
}
