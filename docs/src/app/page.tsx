"use client";

import { useEffect } from "react";

export default function RootRedirect() {
  useEffect(() => {
    const sysLang = (
      typeof navigator !== "undefined" ? navigator.language : ""
    ).toLowerCase();
    const target = sysLang.startsWith("ja") ? "/ja" : "/en";
    window.location.replace(target);
  }, []);

  return (
    <p style={{ padding: "2rem", fontFamily: "system-ui, sans-serif" }}>
      Redirecting to <a href="/en">documentation</a>…
    </p>
  );
}
