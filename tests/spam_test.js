import http from 'k6/http';
import { sleep } from 'k6';
import { Trend, Counter } from 'k6/metrics';

const totalDuration = new Trend('full_cycle_ms', true);
const successCount = new Counter('successful_jobs');
const failCount = new Counter('failed_jobs');

export const options = {
  vus: 10,
  iterations: 100,
};

const payloads = [
  // --- Run submissions (no testcases) ---
  {
    language: 'python',
    version: '3.11',
    code: 'print("Hello from Python 3.11")',
    submission_type: 'Run',
    testcases: [],
  },
  {
    language: 'python',
    version: '3.9',
    code: 'print("Hello from Python 3.9")',
    submission_type: 'Run',
    testcases: [],
  },
  {
    language: 'java',
    version: '25',
    code: 'public class Main {\n    public static void main(String[] args) {\n        System.out.println("Hello from Java 25");\n    }\n}',
    submission_type: 'Run',
    testcases: [],
  },
  // --- Submit submissions (with testcases) ---
  {
    language: 'python',
    version: '3.11',
    code: 'print(input())',
    submission_type: 'Submit',
    testcases: [
      { testcase_id: '1', input: 'Vulkan', expected_output: 'Vulkan' },
      { testcase_id: '2', input: 'Hello', expected_output: 'Hello' },
    ],
  },
  {
    language: 'python',
    version: '3.9',
    code: 'a, b = map(int, input().split())\nprint(a + b)',
    submission_type: 'Submit',
    testcases: [
      { testcase_id: '1', input: '3 5', expected_output: '8' },
      { testcase_id: '2', input: '10 20', expected_output: '30' },
    ],
  },
  {
    language: 'java',
    version: '25',
    code: 'import java.util.Scanner;\npublic class Main {\n    public static void main(String[] args) {\n        Scanner sc = new Scanner(System.in);\n        int a = sc.nextInt(), b = sc.nextInt();\n        System.out.println(a + b);\n    }\n}',
    submission_type: 'Submit',
    testcases: [
      { testcase_id: '1', input: '4 6', expected_output: '10' },
      { testcase_id: '2', input: '1 99', expected_output: '100' },
    ],
  },
];

// Low-priority payload: Submit with 11 testcases (> TESTCASE_COUNT_LIMIT=10 → low queue)
const lowPriorityPayload = {
  language: 'python',
  version: '3.11',
  code: 'print(int(input()) * 2)',
  submission_type: 'Submit',
  testcases: [
    { testcase_id: '1',  input: '1',  expected_output: '2'  },
    { testcase_id: '2',  input: '2',  expected_output: '4'  },
    { testcase_id: '3',  input: '3',  expected_output: '6'  },
    { testcase_id: '4',  input: '4',  expected_output: '8'  },
    { testcase_id: '5',  input: '5',  expected_output: '10' },
    { testcase_id: '6',  input: '6',  expected_output: '12' },
    { testcase_id: '7',  input: '7',  expected_output: '14' },
    { testcase_id: '8',  input: '8',  expected_output: '16' },
    { testcase_id: '9',  input: '9',  expected_output: '18' },
    { testcase_id: '10', input: '10', expected_output: '20' },
    { testcase_id: '11', input: '11', expected_output: '22' },
  ],
};

export default function () {
  // Every 10th iteration (0-indexed: 9, 19, 29, …) → low-priority Submit (11 testcases)
  // Remaining 90 iterations rotate through the 6 normal payloads
  const payload = __ITER % 10 === 9 ? lowPriorityPayload : payloads[__ITER % payloads.length];
  const start = Date.now();

  const submitRes = http.post(
    'http://localhost:8000/execute',
    JSON.stringify(payload),
    { headers: { 'Content-Type': 'application/json' } }
  );

  const jobId = submitRes.body.match(
    /[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/
  )?.[0];

  if (!jobId) {
    console.error(`[VU ${__VU} ITER ${__ITER}] Failed to parse job ID: ${submitRes.body}`);
    failCount.add(1);
    return;
  }

  // Poll until result is ready
  let result;
  while (true) {
    const pollRes = http.get(`http://localhost:8000/job/${jobId}`);
    result = pollRes.json();

    if (result.stderr !== 'Job ID not found') break;
    sleep(0.1);
  }

  const elapsed = Date.now() - start;
  totalDuration.add(elapsed);
  successCount.add(1);

  console.log(`[VU ${__VU} ITER ${__ITER}] lang=${payload.language}@${payload.version} type=${payload.submission_type} | status=${result.status} | stdout=${result.stdout?.trim()} | ${elapsed}ms`);
}
