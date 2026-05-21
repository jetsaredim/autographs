import Link from "next/link";

import { AdminUnlock } from "./AdminUnlock";

export function PublicFooter() {
  return (
    <footer className="public-footer">
      <span>Jared Greenwald&apos;s Autograph Gallery</span>
      <span className="public-footer-links">
        <AdminUnlock />
        <Link href="/architecture">About</Link>
      </span>
    </footer>
  );
}
