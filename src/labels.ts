// Turning a command line into "which project is this?" (PRD §6 step 3).
//
// Working directory would answer this directly but is out of scope (§6 step 4):
// there is no supported API, and the undocumented PEB read fails on exactly the
// elevated processes we already cannot see into. The command line almost always
// carries the project path anyway.

const RUNNERS = [
  "vite",
  "next",
  "webpack",
  "uvicorn",
  "dotnet",
  "supabase",
  "func",
  "nodemon",
  "astro",
  "remix",
  "nest",
  "rollup",
  "parcel",
  "esbuild",
  "gunicorn",
  "flask",
  "django",
  "rails",
];

/// Directories that are never the answer to "which project?".
const NOISE = new Set([
  "bin",
  "node_modules",
  "nodejs",
  "system32",
  "syswow64",
  "windows",
  "program files",
  "program files (x86)",
  // "program" alone catches an unquoted "C:\Program Files\..." clipped at the space.
  "program",
  "programdata",
  "appdata",
  "local",
  "roaming",
  "temp",
  "dist",
  "build",
  "src",
  ".bin",
]);

export function runnerLabel(cmd: string | null): string | null {
  if (!cmd) return null;
  const lower = cmd.toLowerCase();
  return RUNNERS.find((r) => new RegExp(`(^|[\\\\/\\s"'])${r}([\\s"'.]|$)`).test(lower)) ?? null;
}

/**
 * Deepest meaningful folder segment of a path in the command line.
 *
 * The key move is cutting at `node_modules`: a Vite server's command line points
 * at `<project>\node_modules\vite\bin\vite.js`, so the segment immediately before
 * `node_modules` is the project, while the tail is just the runner's own install
 * path and identical for every project on the machine.
 */
export function projectLabel(cmd: string | null): string | null {
  if (!cmd) return null;
  // Quoted paths FIRST and whole: Windows quotes any path containing spaces, and
  // a whitespace-terminated match would clip "RL Finance" to "RL". Bare paths are
  // only a fallback, and can only be trusted up to their first space.
  const quoted = [...cmd.matchAll(/"([A-Za-z]:[\\/][^"]*)"/g)].map((m) => m[1]);
  const bare = cmd.match(/[A-Za-z]:[\\/][^"'\s]*/g) ?? [];
  const paths = [...quoted, ...bare];
  if (paths.length === 0) return null;

  for (const raw of paths) {
    let p = raw.replace(/\//g, "\\");
    const nm = p.toLowerCase().indexOf("\\node_modules\\");
    if (nm > -1) p = p.slice(0, nm);
    else p = p.replace(/\\[^\\]*\.(exe|js|mjs|cjs|cmd|bat|ps1|py|dll)$/i, "");

    const segments = p.split("\\").filter(Boolean);
    for (let i = segments.length - 1; i >= 1; i--) {
      const seg = segments[i];
      if (/^[A-Za-z]:$/.test(seg)) break;
      if (NOISE.has(seg.toLowerCase())) continue;
      if (/\.(exe|js|mjs|cjs|cmd|bat|ps1|py|dll)$/i.test(seg)) continue;
      return seg;
    }
  }
  return null;
}

