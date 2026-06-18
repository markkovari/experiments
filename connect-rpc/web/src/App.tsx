import { useState } from "react";
import { StreamingView } from "./components/StreamingView.js";
import { PollingView } from "./components/PollingView.js";

type Mode = "stream" | "poll";

export function App() {
  const [mode, setMode] = useState<Mode>("stream");

  return (
    <div className="mx-auto max-w-2xl p-8">
      <h1 className="text-2xl font-bold text-slate-800">JobRunner</h1>
      <p className="mb-6 text-slate-500">
        React + Rust over Connect-RPC. Submit a job, then watch progress via{" "}
        <strong>server streaming</strong> or <strong>client polling</strong> —
        same backend, same protocol.
      </p>

      <div className="mb-6 inline-flex rounded-lg border border-slate-200 p-1">
        <Tab active={mode === "stream"} onClick={() => setMode("stream")}>
          Streaming
        </Tab>
        <Tab active={mode === "poll"} onClick={() => setMode("poll")}>
          Polling
        </Tab>
      </div>

      {mode === "stream" ? <StreamingView /> : <PollingView />}
    </div>
  );
}

function Tab({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      className={
        "rounded-md px-4 py-1.5 text-sm font-medium transition " +
        (active ? "bg-slate-800 text-white" : "text-slate-600")
      }
    >
      {children}
    </button>
  );
}
