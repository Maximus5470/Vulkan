import http from 'k6/http';
import { sleep } from 'k6';
import { Trend } from 'k6/metrics';

const totalDuration = new Trend('full_cycle_ms', true);

export const options = {
  vus: 1,
  iterations: 1,
};

export default function () {
  const start = Date.now();

  // Step 1: Submit job
  const submitRes = http.post(
    'http://localhost:8000/execute',
    JSON.stringify({
      language: 'java',
      version: '25',
      code: "public class Main {\n    public static void main(String[] args) {\n        int n = Integer.parseInt(\"5\");\n        System.out.println(n * 2);\n    }\n}",
      submission_type: 'Run',
      testcases: [],
    }),
    { headers: { 'Content-Type': 'application/json' } }
  );

  const jobId = submitRes.body.match(
    /[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/
  )?.[0];

  if (!jobId) {
    console.error('Failed to parse job ID:', submitRes.body);
    return;
  }

  // Step 2: Poll until result is ready
  let result;
  while (true) {
    const pollRes = http.get(`http://localhost:8000/job/${jobId}`);
    result = pollRes.json();

    if (result.stderr !== 'Job ID not found') break;
    sleep(0.1);
  }

  const elapsed = Date.now() - start;
  totalDuration.add(elapsed);

  console.log(`Status: ${result.status} | Stdout: ${result.stdout} | Total: ${elapsed}ms`);
}