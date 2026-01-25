// Windmill TypeScript script for distributed factorial calculation
// This script orchestrates factorial calculations using NATS workers

import { connect, StringCodec, JSONCodec } from "npm:nats@2.28.2";

type Input = {
  number: number;
  request_id?: string;
};

type Output = {
  number: number;
  result: string;
  cache_hit: boolean;
  worker_id: string;
  job_id: string;
};

type FactorialRequest = {
  number: number;
  request_id: string;
  original_request: number;
};

type FactorialResponse = {
  number: number;
  request_id: string;
  result: string;
  error?: string;
};

export async function main(input: Input): Promise<Output> {
  const jobId = Deno.env.get("WM_JOB_ID") || "unknown";
  const requestId = input.request_id || `windmill-${jobId}`;

  console.log(
    `[Windmill Job ${jobId}] Starting factorial calculation for ${input.number}`
  );

  // Connect to NATS
  const natsUrl = Deno.env.get("NATS_URL") || "nats://nats:4222";
  const nc = await connect({ servers: natsUrl });
  console.log(`[Windmill Job ${jobId}] Connected to NATS at ${natsUrl}`);

  const jc = JSONCodec<FactorialRequest | FactorialResponse>();

  try {
    // Make request to NATS workers
    const request: FactorialRequest = {
      number: input.number,
      request_id: requestId,
      original_request: input.number,
    };

    console.log(
      `[Windmill Job ${jobId}] Sending request to NATS workers:`,
      request
    );

    // Use NATS request-response pattern
    const msg = await nc.request(
      "factorial.request",
      jc.encode(request),
      { timeout: 30000 } // 30 second timeout
    );

    const response = jc.decode(msg.data) as FactorialResponse;

    console.log(
      `[Windmill Job ${jobId}] Received response from worker:`,
      response
    );

    if (response.error) {
      throw new Error(`Worker error: ${response.error}`);
    }

    await nc.close();

    return {
      number: input.number,
      result: response.result,
      cache_hit: false, // Workers handle caching internally
      worker_id: "distributed",
      job_id: jobId,
    };
  } catch (error) {
    await nc.close();
    throw error;
  }
}
