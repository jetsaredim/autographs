import Link from "next/link";

import { AdminUnlock } from "./AdminUnlock";

export function PublicFooter() {
  return (
    <footer className="public-footer">
      <span>Jared Greenwald&apos;s Autograph Gallery</span>
      <span aria-hidden="true">•</span>
      <Link href="/architecture">About</Link>
      <AdminUnlock />
    </footer>
  );
}
