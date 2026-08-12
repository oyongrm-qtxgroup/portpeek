import { getCurrentWindow } from "@tauri-apps/api/window";

// ponytail: M1 is the window shell only. These rows are static fixtures that
// exercise every row variant the real list will render (normal / shielded /
// forwarded), so the M2 swap to live data is a data-source change, not a redesign.
type MockRow = {
  port: number;
  process: string;
  label?: string;
  cmd?: string;
  shield?: "hard" | "forwarded";
  badge?: string;
};

const MOCK_ROWS: MockRow[] = [
  {
    port: 1420,
    process: "node",
    label: "vite",
    cmd: "C:\\Users\\Raymundo Oyong\\Documents\\Projects\\Port Watcher> vite",
  },
  {
    port: 3000,
    process: "node",
    label: "next",
    cmd: "C:\\projects\\jayne-dashboard> next dev",
  },
  { port: 445, process: "System", shield: "hard", badge: "HTTP.sys / Windows" },
  {
    port: 50505,
    process: "com.docker.backend.exe",
    shield: "forwarded",
    badge: "Docker",
  },
];

const FORWARDED_TIP =
  "Forwarded from Docker/WSL. Stop the container or WSL service instead — killing this relay won't stop the underlying server and may break other forwards.";

function Header({ count }: { count: number }) {
  // data-tauri-drag-region applies only to the element it sits on, so the
  // non-interactive children carry it too — otherwise dragging by the title fails.
  return (
    <header
      data-tauri-drag-region
      className="flex select-none items-center gap-2 border-b border-white/10 px-3 py-2"
    >
      <span data-tauri-drag-region className="text-sm font-semibold text-zinc-100">
        PortPeek
      </span>
      <span
        data-tauri-drag-region
        className="rounded-full bg-white/10 px-1.5 py-0.5 font-mono text-[10px] text-zinc-300"
      >
        {count}
      </span>
      <span data-tauri-drag-region className="flex-1" />
      <button
        title="Refresh now"
        className="rounded px-1.5 py-0.5 text-zinc-400 hover:bg-white/10 hover:text-zinc-100"
      >
        ⟳
      </button>
      <button
        // hide(), not minimize(): minimising a frameless always-on-top widget leaves
        // a taskbar button and no window. Hiding parks it wholly in the tray.
        title="Hide to tray"
        onClick={() => getCurrentWindow().hide()}
        className="rounded px-1.5 py-0.5 text-zinc-400 hover:bg-white/10 hover:text-zinc-100"
      >
        −
      </button>
      <button
        title="Close PortPeek"
        onClick={() => getCurrentWindow().close()}
        className="rounded px-1.5 py-0.5 text-zinc-400 hover:bg-red-500/80 hover:text-white"
      >
        ✕
      </button>
    </header>
  );
}

function Favorites() {
  return (
    <div className="flex items-center gap-1.5 border-b border-white/10 px-3 py-2">
      <span className="flex items-center gap-1.5 rounded-md bg-white/5 px-2 py-1 font-mono text-xs text-zinc-200">
        <span className="size-1.5 rounded-full bg-amber-400" />
        8080
        <span className="font-sans text-[10px] text-zinc-400">node</span>
        <button
          title="Free port 8080"
          className="ml-0.5 text-zinc-500 hover:text-red-400"
        >
          ✕
        </button>
      </span>
      <button
        title="Add a favorite port"
        className="rounded-md px-1.5 py-1 text-xs text-zinc-500 hover:bg-white/10 hover:text-zinc-200"
      >
        +
      </button>
    </div>
  );
}

function Row({ row }: { row: MockRow }) {
  return (
    <li className="group flex items-start gap-2 px-3 py-1.5 hover:bg-white/5">
      <span
        className={`mt-1.5 size-1.5 shrink-0 rounded-full ${
          row.shield ? "bg-zinc-600" : "bg-emerald-400"
        }`}
      />
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-1.5">
          <span className="font-mono text-xs font-bold text-zinc-100">
            :{row.port}
          </span>
          <span className="truncate text-xs text-zinc-300">{row.process}</span>
          {row.label && (
            <span className="text-xs text-zinc-500">— {row.label}</span>
          )}
          {row.badge && (
            <span
              title={row.shield === "forwarded" ? FORWARDED_TIP : undefined}
              className="shrink-0 rounded bg-sky-500/15 px-1 py-px text-[9px] font-medium text-sky-300"
            >
              {row.badge}
            </span>
          )}
        </div>
        {row.cmd && (
          <div title={row.cmd} className="truncate text-[10px] text-zinc-500">
            {row.cmd}
          </div>
        )}
      </div>
      {row.shield ? (
        <span
          title={
            row.shield === "hard"
              ? "Protected system process — cannot be terminated."
              : FORWARDED_TIP
          }
          className="mt-0.5 shrink-0 text-xs text-zinc-600"
        >
          ⛊
        </span>
      ) : (
        <button
          title={`Free port ${row.port}`}
          className="mt-0.5 shrink-0 rounded px-1 text-xs text-zinc-600 opacity-0 hover:bg-red-500/80 hover:text-white group-hover:opacity-100"
        >
          ✕
        </button>
      )}
    </li>
  );
}

export default function App() {
  return (
    // The p-2 gutter is deliberate: it is the transparency probe. If the window
    // is genuinely transparent you see the desktop around the rounded card;
    // if the Tauri/WebView2 regression bites, this gutter renders as a solid block.
    <div className="h-full p-2">
      <div className="flex h-full flex-col overflow-hidden rounded-xl border border-white/10 bg-zinc-900/85 text-zinc-200 shadow-2xl backdrop-blur-md">
        <Header count={MOCK_ROWS.length} />
        <Favorites />
        <ul className="flex-1 overflow-y-auto py-1">
          {MOCK_ROWS.map((row) => (
            <Row key={`${row.port}-${row.process}`} row={row} />
          ))}
        </ul>
      </div>
    </div>
  );
}
