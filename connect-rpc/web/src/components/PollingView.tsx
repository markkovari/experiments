import { useEffect, useRef, useState } from "react";
import { create } from "@bufbuild/protobuf";
import { client } from "../client.js";
import {
  JobPhase,
  JobRequestSchema,
  JobHandleSchema,
  type JobStatus,
} from "../gen/jobrunner/v1/jobrunner_pb.js";

// Client-pull path: StartJob kicks off the job, then we poll GetJobStatus
// on a timer until the job reports DONE.
export function PollingView() {
  const [label, setLabel] = useState("backup");
  const [steps, setSteps] = useState(8);
  const [status, setStatus] = useState<JobStatus | null>(null);
  const [polling, setPolling] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const timer = useRef<number | null>(null);

  // Clean up the interval if the component unmounts mid-poll.
  useEffect(() => () => clearTimer(), []);

  function clearTimer() {
    if (timer.current !== null) {
      clearInterval(timer.current);
      timer.current = null;
    }
  }

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    clearTimer();
    setStatus(null);
    setError(null);
    setPolling(true);
    try {
      const handle = await client.startJob(
        create(JobRequestSchema, { label, steps }),
      );
      timer.current = window.setInterval(async () => {
        try {
          const s = await client.getJobStatus(
            create(JobHandleSchema, { id: handle.id }),
          );
          setStatus(s);
          if (s.phase === JobPhase.DONE) {
            clearTimer();
            setPolling(false);
          }
        } catch (err) {
          clearTimer();
          setPolling(false);
          setError(err instanceof Error ? err.message : String(err));
        }
      }, 500);
    } catch (err) {
      setPolling(false);
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  const pct =
    status && status.total > 0 ? (status.step / status.total) * 100 : 0;

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
          disabled={polling}
          className="rounded bg-emerald-600 px-4 py-1.5 font-medium text-white disabled:opacity-50"
        >
          {polling ? "Polling…" : "Run (poll)"}
        </button>
      </form>

      {error && <p className="text-red-600">{error}</p>}

      <div className="h-2 w-full overflow-hidden rounded bg-slate-200">
        <div
          className="h-full bg-emerald-500 transition-all"
          style={{ width: `${pct}%` }}
        />
      </div>

      {status && (
        <div className="font-mono text-sm">
          <p>
            phase: <span className="font-semibold">{phaseName(status.phase)}</span>
          </p>
          <p>
            [{status.step}/{status.total}] {status.message}
          </p>
        </div>
      )}
    </div>
  );
}

function phaseName(p: JobPhase): string {
  switch (p) {
    case JobPhase.RUNNING:
      return "RUNNING";
    case JobPhase.DONE:
      return "DONE";
    default:
      return "UNSPECIFIED";
  }
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
