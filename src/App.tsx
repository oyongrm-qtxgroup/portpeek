import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { projectLabel, runnerLabel } from "./labels";

type PortRow = {
  port: number;
  pid: number;
  process: string;
  exe: string | null;
  cmd: string | null;
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

  return { rows, error, loaded, refresh, setError };
}

function useAutostart(onError: (msg: string) => void) {
  const [on, setOn] = useState(false);

  useEffect(() => {
    invoke<boolean>("get_autostart").then(setOn).catch(() => {});
  }, []);

  const toggle = useCallback(async () => {
    try {
      // The command returns the state read back from the OS, so `on` can never
      // drift from what actually got written.
      setOn(await invoke<boolean>("set_autostart", { enabled: !on }));
    } catch (e) {
      onError(String(e));
    }
  }, [on, onError]);

  return { on, toggle };
}

function Header({
  count,
  onRefresh,
  autostart,
}: {
  count: number;
  onRefresh: () => void;
  autostart: ReturnType<typeof useAutostart>;
}) {
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
        title={
          autostart.on
            ? "Start with Windows: ON — click to disable"
            : "Start with Windows: OFF — click to enable"
        }
        onClick={autostart.toggle}
        className={`rounded px-1.5 py-0.5 hover:bg-white/10 ${
          autostart.on ? "text-amber-400" : "text-zinc-600 hover:text-zinc-100"
        }`}
      >
        ⏻
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
  const runner = runnerLabel(row.cmd);
  const project = projectLabel(row.cmd);
  // Degrade, never blank: cmd -> exe -> a plain statement that we cannot see in.
  const detail = row.cmd ?? row.exe ?? "no command line — run as administrator for detail";
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
          {runner && (
            <span className="shrink-0 text-xs text-zinc-500">— {runner}</span>
          )}
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
          title={detail}
          className={`truncate text-[10px] ${row.cmd ? "text-zinc-500" : "text-zinc-600 italic"}`}
        >
          {project && (
            <span className="not-italic font-medium text-zinc-400">{project} · </span>
          )}
          {detail}
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
  const { rows, error, loaded, refresh, setError } = usePorts();
  const autostart = useAutostart(setError);
  const [filter, setFilter] = useState("");

  const q = filter.trim().toLowerCase();
  const shown = q
    ? rows.filter((r) =>
        [String(r.port), r.process, r.cmd ?? "", projectLabel(r.cmd) ?? ""]
          .join(" ")
          .toLowerCase()
          .includes(q),
      )
    : rows;

  // PID 0/4 never expose a command line by design, so they must not trigger a
  // hint implying the user is missing something fixable.
  const denied = rows.filter((r) => r.pid > 4 && !r.cmd).length;

  return (
    <div className="flex h-full flex-col overflow-hidden bg-zinc-900 text-zinc-200">
      <Header count={rows.length} onRefresh={refresh} autostart={autostart} />
      <Favorites rows={rows} />
      <div className="border-b border-white/10 px-3 py-1.5">
        <input
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          placeholder="Filter port, process, project…"
          spellCheck={false}
          className="w-full rounded bg-white/5 px-2 py-1 text-[11px] text-zinc-200 placeholder:text-zinc-600 focus:bg-white/10 focus:outline-none"
        />
      </div>
      {denied > 0 && (
        <div
          title="Windows returns an empty command line for processes owned by another user or running elevated. This is a permission limit, not an error."
          className="border-b border-amber-500/20 bg-amber-500/5 px-3 py-1 text-[10px] text-amber-300/80"
        >
          {denied} process{denied === 1 ? "" : "es"} hidden detail — run as
          administrator for full command lines
        </div>
      )}
      {error && (
        <div className="border-b border-red-500/30 bg-red-500/10 px-3 py-1.5 text-[10px] text-red-300">
          {error}
        </div>
      )}
      <ul className="flex-1 overflow-y-auto py-1">
        {shown.map((row) => (
          <Row key={`${row.port}-${row.pid}`} row={row} />
        ))}
        {loaded && shown.length === 0 && !error && (
          <li className="px-3 py-4 text-center text-xs text-zinc-500">
            {rows.length === 0
              ? "No TCP listeners found."
              : `Nothing matches “${filter}”.`}
          </li>
        )}
      </ul>
    </div>
  );
}
