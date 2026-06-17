import { useState } from "react";
import { create } from "@bufbuild/protobuf";
import { client } from "../client.js";
import {
  JobRequestSchema,
  type JobUpdate,
} from "../gen/jobrunner/v1/jobrunner_pb.js";

// Server-push path: a single server-streaming RPC. We iterate the async
// stream and render each JobUpdate as it arrives.
export function StreamingView() {
  const [label, setLabel] = useState("deploy");
  const [steps, setSteps] = useState(8);
  const [updates, setUpdates] = useState<JobUpdate[]>([]);
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setUpdates([]);
    setError(null);
    setRunning(true);
    try {
      const req = create(JobRequestSchema, { label, steps });
      for await (const update of client.runJobStreaming(req)) {
        setUpdates((prev) => [...prev, update]);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setRunning(false);
    }
  }

  const last = updates[updates.length - 1];
  const pct = last && last.total > 0 ? (last.step / last.total) * 100 : 0;

  return (
    <div className="space-y-4">
      <form onSubmit={onSubmit} className="flex flex-wrap items-end gap-3">
        <Field label="Label">
          <input
            className="rounded border border-slate-300 px-3 py-1.5"
            value={label}
            onChange={(e) => setLabel(e.target.value)}
          />
        </Field>
        <Field label="Steps">
          <input
            type="number"
            min={1}
            max={50}
            className="w-24 rounded border border-slate-300 px-3 py-1.5"
            value={steps}
            onChange={(e) => setSteps(Number(e.target.value))}
          />
        </Field>
        <button
          type="submit"
          disabled={running}
          className="rounded bg-indigo-600 px-4 py-1.5 font-medium text-white disabled:opacity-50"
        >
          {running ? "Streaming…" : "Run (stream)"}
        </button>
      </form>

      {error && <p className="text-red-600">{error}</p>}

      <div className="h-2 w-full overflow-hidden rounded bg-slate-200">
        <div
          className="h-full bg-indigo-500 transition-all"
          style={{ width: `${pct}%` }}
        />
      </div>

      <ul className="space-y-1 font-mono text-sm">
        {updates.map((u, i) => (
          <li key={i} className={u.done ? "text-green-700" : "text-slate-700"}>
            [{u.step}/{u.total}] {u.message}
            {u.done ? " ✓" : ""}
          </li>
        ))}
      </ul>
    </div>
  );
}

function Field({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <label className="flex flex-col gap-1 text-sm">
      <span className="text-slate-500">{label}</span>
      {children}
    </label>
  );
}
