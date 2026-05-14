import { createOracleExecutor } from "../src/db/oracle";
import { runMigrations } from "../src/db/migrations";

const executor = createOracleExecutor();
const applied = await runMigrations(executor);

for (const migration of applied) {
  console.log(`applied ${migration.version} ${migration.name}`);
}

if (applied.length === 0) {
  console.log("database schema already up to date");
}
