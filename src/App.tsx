import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

type PortRow = {
  port: number;
  pid: number;
  process: string;
  exe: string | null;
  ipv4: boolean;
  ipv6: boolean;
  shield: "hard" | "critical" | null;
  badge: string | null;
};

function usePorts() {
  const [rows, setRows] = useState<PortRow[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loaded, setLoaded] = useState(false);

  // Manual refresh only. The 2s cadence is driven from Rust and arrives as an
  // event, because a JS setInterval gets throttled when the window is occluded.
  const refresh = useCallback(async () => {
    try {
      setRows(await invoke<PortRow[]>("list_ports"));
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoaded(true);
    }
  }, []);

  useEffect(() => {
    refresh();
    const unlisten = Promise.all([
      listen<PortRow[]>("ports", (e) => {
        setRows(e.payload);
        setError(null);
        setLoaded(true);
      }),
      listen<string>("ports-error", (e) => {
        setError(e.payload);
        setLoaded(true);
      }),
    ]);
    return () => {
      unlisten.then((fns) => fns.forEach((f) => f()));
    };
  }, [refresh]);

  return { rows, error, loaded, refresh };
}

function Header({ count, onRefresh }: { count: number; onRefresh: () => void }) {
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
        onClick={onRefresh}
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

// ponytail: favourites are hard-coded to 8080 until M5 adds add/remove +
// persistence. The status is already live so the chip never shows a stale lie.
function Favorites({ rows }: { rows: PortRow[] }) {
  const holder = rows.find((r) => r.port === 8080);
  return (
    <div className="flex items-center gap-1.5 border-b border-white/10 px-3 py-2">
      <span className="flex items-center gap-1.5 rounded-md bg-white/5 px-2 py-1 font-mono text-xs text-zinc-200">
        <span
          className={`size-1.5 rounded-full ${holder ? "bg-amber-400" : "bg-emerald-400"}`}
        />
        8080
        <span className="font-sans text-[10px] text-zinc-400">
          {holder ? holder.process : "free"}
        </span>
      </span>
    </div>
  );
}

function Row({ row }: { row: PortRow }) {
  const stack = row.ipv4 && row.ipv6 ? "v4+v6" : row.ipv6 ? "v6" : "v4";
  return (
    <li className="group flex items-start gap-2 px-3 py-1.5 hover:bg-white/5">
      <span
        className={`mt-1.5 size-1.5 shrink-0 rounded-full ${
          row.shield === "hard"
            ? "bg-zinc-600"
            : row.shield === "critical"
              ? "bg-red-500"
              : "bg-emerald-400"
        }`}
      />
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-1.5">
          <span className="font-mono text-xs font-bold text-zinc-100">
            :{row.port}
          </span>
          <span className="truncate text-xs text-zinc-300">{row.process}</span>
          <span className="shrink-0 font-mono text-[9px] text-zinc-600">
            {stack}
          </span>
          {row.badge && (
            <span className="shrink-0 rounded bg-sky-500/15 px-1 py-px text-[9px] font-medium text-sky-300">
              {row.badge}
            </span>
          )}
        </div>
        <div
          title={row.exe ?? undefined}
          className="truncate text-[10px] text-zinc-500"
        >
          pid {row.pid}
          {row.exe ? ` · ${row.exe}` : ""}
        </div>
      </div>
      {row.shield === "hard" ? (
        <span
          title={
            row.pid === 4
              ? "Protected: PID 4 (System). http.sys owns this listener — find the real owner with: netsh http show servicestate"
              : "Protected system process — cannot be terminated."
          }
          className="mt-0.5 shrink-0 text-xs text-zinc-600"
        >
          ⛊
        </span>
      ) : (
        // ponytail: kill wiring lands in M4; the affordance is here so the shield
        // rules are visible and testable now.
        <button
          disabled
          title={
            row.shield === "critical"
              ? "Critical system process — terminating it can bugcheck Windows (kill flow lands in M4)"
              : "Free this port (kill flow lands in M4)"
          }
          className={`mt-0.5 shrink-0 rounded px-1 text-xs opacity-0 group-hover:opacity-100 ${
            row.shield === "critical" ? "text-red-400" : "text-zinc-600"
          }`}
        >
          ✕
        </button>
      )}
    </li>
  );
}

export default function App() {
  const { rows, error, loaded, refresh } = usePorts();

  return (
    <div className="flex h-full flex-col overflow-hidden bg-zinc-900 text-zinc-200">
      <Header count={rows.length} onRefresh={refresh} />
      <Favorites rows={rows} />
      {error && (
        <div className="border-b border-red-500/30 bg-red-500/10 px-3 py-1.5 text-[10px] text-red-300">
          {error}
        </div>
      )}
      <ul className="flex-1 overflow-y-auto py-1">
        {rows.map((row) => (
          <Row key={`${row.port}-${row.pid}`} row={row} />
        ))}
        {loaded && rows.length === 0 && !error && (
          <li className="px-3 py-4 text-center text-xs text-zinc-500">
            No TCP listeners found.
          </li>
        )}
      </ul>
    </div>
  );
}
