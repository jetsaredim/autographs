import Link from "next/link";

export function PublicFooter() {
  return (
    <footer className="public-footer">
      <span>Jared Greenwald&apos;s Autograph Gallery</span>
      <Link href="/architecture">About</Link>
    </footer>
  );
}
