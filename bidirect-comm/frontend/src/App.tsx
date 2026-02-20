import { useState, useEffect, useRef } from "react";
import "./App.css";

// --- HTTP Panel ---
function HttpPanel({ path, label }: { path: string; label: string }) {
  const [input, setInput] = useState("");
  const [result, setResult] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const submit = async () => {
    const n = parseInt(input, 10);
    if (isNaN(n) || n < 0) return;
    setLoading(true);
    try {
      const res = await fetch(`http://localhost:3001/${path}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ n }),
      });
      const data = await res.json() as { n: number; result: string; error?: string };
      if (data.error) setResult(`Error: ${data.error}`);
      else setResult(`${label}(${data.n}) = ${data.result}`);
    } catch {
      setResult("Connection error");
    } finally {
      setLoading(false);
    }
  };

  return (
    <section className="panel">
      <h2>HTTP</h2>
      <p className="description">Send a number, get {label}(n) back in one response.</p>
      <div className="input-row">
        <input
          type="number"
          min={0}
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && submit()}
          placeholder="Enter n"
        />
        <button onClick={submit} disabled={loading}>
          {loading ? "…" : "Send"}
        </button>
      </div>
      {result && <p className="result">{result}</p>}
    </section>
  );
}

// --- WebSocket Panel ---
function WsPanel({ type, label }: { type: string; label: string }) {
  const [input, setInput] = useState("");
  const [messages, setMessages] = useState<string[]>([]);
  const wsRef = useRef<WebSocket | null>(null);
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    const ws = new WebSocket("ws://localhost:3002");
    wsRef.current = ws;

    ws.onopen = () => setConnected(true);
    ws.onclose = () => setConnected(false);
    ws.onmessage = (e) => {
      const data = JSON.parse(e.data as string) as { type: string; n: number; result: string; error?: string };
      if (data.error) {
        setMessages((prev) => [...prev, `Error: ${data.error}`]);
      } else if (data.type === type) {
        setMessages((prev) => [...prev, `${label}(${data.n}) = ${data.result}`]);
      }
    };

    return () => ws.close();
  }, [type, label]);

  const send = () => {
    const n = parseInt(input, 10);
    if (isNaN(n) || n < 0 || !wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) return;
    wsRef.current.send(JSON.stringify({ n, type }));
    setInput("");
  };

  return (
    <section className="panel">
      <h2>WebSocket <span className={`dot ${connected ? "green" : "red"}`} /></h2>
      <p className="description">Bidirectional — send multiple values, results arrive as they come.</p>
      <div className="input-row">
        <input
          type="number"
          min={0}
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && send()}
          placeholder="Enter n"
          disabled={!connected}
        />
        <button onClick={send} disabled={!connected}>Send</button>
      </div>
      <ul className="message-list">
        {messages.map((m, i) => <li key={i}>{m}</li>)}
      </ul>
    </section>
  );
}

// --- SSE Panel ---
function SsePanel({ path, label }: { path: string; label: string }) {
  const [input, setInput] = useState("");
  const [stream, setStream] = useState<string[]>([]);
  const [streaming, setStreaming] = useState(false);

  const startStream = () => {
    const n = parseInt(input, 10);
    if (isNaN(n) || n < 0) return;

    setStream([]);
    setStreaming(true);

    const es = new EventSource(`http://localhost:3003/${path}/stream?n=${n}`);

    es.onmessage = (e) => {
      if ((e.data as string) === "[DONE]") {
        es.close();
        setStreaming(false);
        return;
      }
      const data = JSON.parse(e.data as string) as { index: number; value: string };
      setStream((prev) => [...prev, `${label}(${data.index}) = ${data.value}`]);
    };

    es.onerror = () => {
      es.close();
      setStreaming(false);
    };
  };

  return (
    <section className="panel">
      <h2>Server-Sent Events</h2>
      <p className="description">Streams {label}(0) through {label}(n) back one value at a time.</p>
      <div className="input-row">
        <input
          type="number"
          min={0}
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && startStream()}
          placeholder="Enter n"
          disabled={streaming}
        />
        <button onClick={startStream} disabled={streaming}>
          {streaming ? "Streaming…" : "Stream"}
        </button>
      </div>
      <ul className="message-list">
        {stream.map((m, i) => <li key={i}>{m}</li>)}
      </ul>
    </section>
  );
}

export default function App() {
  return (
    <main>
      <section className="group">
        <h2 className="group-title">Fibonacci</h2>
        <div className="panels">
          <HttpPanel path="fibonacci" label="fib" />
          <WsPanel type="fibonacci" label="fib" />
          <SsePanel path="fibonacci" label="fib" />
        </div>
      </section>

      <section className="group">
        <h2 className="group-title">Factorial</h2>
        <div className="panels">
          <HttpPanel path="factorial" label="fact" />
          <WsPanel type="factorial" label="fact" />
          <SsePanel path="factorial" label="fact" />
        </div>
      </section>
    </main>
  );
}
