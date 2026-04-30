export function GET() {
  return Response.json({
    ok: true,
    service: "autographs",
    scope: "proof-of-life",
  });
}
