import { createCatalogRepository } from "../../../src/catalog";
import { getOracleDatabaseConfig } from "../../../src/db/config";
import { createPrivateMediaStore } from "../../../src/media";
import { getMediaStorageConfig } from "../../../src/media/config";

export const dynamic = "force-dynamic";

export async function GET(request: Request) {
  const url = new URL(request.url);
  const live = url.searchParams.get("live") === "1";
  const checks = {
    oracleConfig: check(() => getOracleDatabaseConfig()),
    mediaConfig: check(() => getMediaStorageConfig()),
  };

  if (!live) {
    return dataConfigResponse(checks);
  }

  const authResponse = authorizeOperator(request);
  if (authResponse) {
    return authResponse;
  }

  try {
    await createCatalogRepository().list({ includeUnpublished: true });
    await createPrivateMediaStore().assertReady();
    return Response.json({
      ok: true,
      service: "autographs",
      scope: "data-live",
      checks,
    });
  } catch (error) {
    return Response.json(
      {
        ok: false,
        service: "autographs",
        scope: "data-live",
        error: error instanceof Error ? error.message : "Unknown data/media readiness error",
        checks,
      },
      { status: 503 },
    );
  }
}

const dataConfigResponse = (checks: DataHealthChecks): Response => {
  const ok = checks.oracleConfig.ok && checks.mediaConfig.ok;
  const body = {
    ok,
    service: "autographs",
    scope: "data-config",
  };

  if (process.env.NODE_ENV === "production") {
    return Response.json(body, { status: ok ? 200 : 503 });
  }

  return Response.json({
    ...body,
    checks,
  });
};

const check = (work: () => unknown): { ok: true } | { ok: false; error: string } => {
  try {
    work();
    return { ok: true };
  } catch (error) {
    return {
      ok: false,
      error: error instanceof Error ? error.message : "Unknown configuration error",
    };
  }
};

type DataHealthChecks = {
  oracleConfig: ReturnType<typeof check>;
  mediaConfig: ReturnType<typeof check>;
};

const authorizeOperator = (request: Request): Response | null => {
  const token = process.env.AUTOGRAPHS_OPERATOR_API_TOKEN;
  if (!token) {
    return Response.json({ error: "Live data health is disabled" }, { status: 404 });
  }

  const providedToken = request.headers.get("authorization")?.replace(/^Bearer\s+/i, "");
  if (providedToken !== token) {
    return Response.json({ error: "Unauthorized" }, { status: 401 });
  }

  return null;
};
