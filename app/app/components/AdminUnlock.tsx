"use client";

import Link from "next/link";
import { useEffect, useState } from "react";

const unlockSequence = "gallery";

export function AdminUnlock() {
  const [progress, setProgress] = useState("");
  const [revealed, setRevealed] = useState(false);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.altKey || event.ctrlKey || event.metaKey) {
        return;
      }

      const key = event.key.toLowerCase();
      if (key.length !== 1) {
        return;
      }

      setProgress((current) => {
        const next = `${current}${key}`;
        const suffix = unlockSequence.slice(0, next.length);
        const normalized = unlockSequence.startsWith(next) ? next : key;

        if (normalized === unlockSequence) {
          setRevealed(true);
          return "";
        }

        return suffix === normalized ? normalized : "";
      });
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  if (!revealed) {
    return null;
  }

  return (
    <Link className="admin-unlock" href="/admin">
      Open collection management placeholder
    </Link>
  );
}
