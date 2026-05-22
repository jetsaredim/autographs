import Link from "next/link";

import { AdminUnlock } from "./AdminUnlock";

export function PublicFooter() {
  return (
    <footer className="public-footer">
      <AdminUnlock />
      <Link href="/architecture">About</Link>
    </footer>
  );
}
