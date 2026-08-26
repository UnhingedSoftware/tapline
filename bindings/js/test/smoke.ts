/**
 * One test file, three runtimes.
 *
 * Written as a plain script rather than against `bun:test`, `node:test` or
 * `Deno.test`, because the thing most worth testing here is that the *same*
 * code behaves the same way in all three — and a test that imports a
 * runtime-specific harness cannot check that.
 *
 * ```sh
 * bun  test/smoke.ts
 * node test/smoke.ts
 * deno run --allow-ffi --allow-env --allow-read --unstable-ffi test/smoke.ts
 * ```
 *
 * Network tests are opt-in: set TAPLINE_LIVE=1.
 */

import { detectRuntime } from "../src/ffi.ts";
import {
  concurrency,
  downloadWorkshopItem,
  install,
  Job,
  plan,
  version,
} from "../src/index.ts";

let failures = 0;
let ran = 0;

async function test(name: string, body: () => void | Promise<void>) {
  ran += 1;
  try {
    await body();
    console.log(`  ok   ${name}`);
  } catch (error) {
    failures += 1;
    console.log(`  FAIL ${name}`);
    console.log(`       ${error instanceof Error ? error.message : error}`);
  }
}

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

function assertEquals(actual: unknown, expected: unknown, message?: string) {
  if (actual !== expected) {
    throw new Error(message ?? `expected ${expected}, got ${actual}`);
  }
}

const runtime = detectRuntime();
console.log(`tapline bindings smoke test on ${runtime}`);

await test("the native library loads and reports a version", async () => {
  const v = await version();
  assert(/^\d+\.\d+\.\d+/.test(v), `not a version: ${v}`);
});

await test("a job is awaitable, iterable and cancellable at once", () => {
  // The shape claim from the README, checked rather than asserted in prose.
  const job = plan({ app: 4020, dir: "/nonexistent-on-purpose" });
  assert(job instanceof Job, "not a Job");
  assert(typeof job.then === "function", "not awaitable");
  assert(typeof job.callback === "function", "no callback form");
  assert(typeof job[Symbol.asyncIterator] === "function", "not iterable");
  assert(typeof job.cancel === "function", "not cancellable");
  job.cancel();
  assertEquals(job.cancelled, true, "cancel did not take");
});

await test("a job nobody awaits does not crash the process", async () => {
  // Constructing a job starts it. A caller who cancels one and never looks at
  // it again must not take the process down with an unhandled rejection.
  const job = plan({ app: 4020, dir: "/tmp" });
  job.cancel();
  await new Promise((resolve) => setTimeout(resolve, 50));
});

await test("cancelling before the job starts rejects rather than hanging", async () => {
  const job = plan({ app: 4020, dir: "/tmp" });
  job.cancel();
  let message = "";
  try {
    await job;
  } catch (error) {
    message = error instanceof Error ? error.message : String(error);
  }
  assert(message.includes("cancelled"), `unexpected: ${message}`);
});

await test("a callback-style consumer gets the same failure", async () => {
  const job = plan({ app: 4020, dir: "/tmp" });
  job.cancel();
  const message = await new Promise<string>((resolve) => {
    job.callback((error) => resolve(error ? error.message : "no error"));
  });
  assert(message.includes("cancelled"), `unexpected: ${message}`);
});

if (process_env("TAPLINE_LIVE") === "1") {
  await test("plan reports a real app's cost without downloading it", async () => {
    const report = await plan({ app: 4020, dir: scratch("plan-only") });
    assert(report.totalBytes > 1_000_000_000, `too small: ${report.totalBytes}`);
    assert(report.fileCount > 2_000, `too few files: ${report.fileCount}`);
    assert(report.chunkCount > 0, "no chunks");
  });

  await test("an app that does not exist fails with a message", async () => {
    let message = "";
    try {
      await plan({ app: 1, dir: scratch("bogus") });
    } catch (error) {
      message = error instanceof Error ? error.message : String(error);
    }
    assert(message.length > 0, "a missing app resolved successfully");
  });

  await test("progress arrives, in order, and ends with a report", async () => {
    const seen: string[] = [];
    let lastDone = 0;
    let monotonic = true;
    let percentInRange = true;

    const job = downloadWorkshopItem({
      app: 4000,
      item: 3790437566n,
      dir: scratch("workshop"),
      onEvent: (event) => seen.push(event.kind),
      onProgress: (p) => {
        if (p.bytesDone < lastDone) monotonic = false;
        lastDone = p.bytesDone;
        if (p.percent < 0 || p.percent > 100) percentInRange = false;
      },
    });

    const report = await job;
    assertEquals(seen[0], "planned", `first event was ${seen[0]}`);
    assert(monotonic, "progress went backwards");
    assert(percentInRange, "percent left 0..100");
    assert(report.files > 0, "no files installed");
    assertEquals(report.skipped.length, 0, "files were skipped");
  });

  await test("the async iterator sees the same events", async () => {
    const kinds: string[] = [];
    for await (const event of plan({ app: 4020, dir: scratch("iter") })) {
      kinds.push(event.kind);
    }
    assert(kinds.includes("planned"), `no planned event: ${kinds.join(",")}`);
  });

  await test("concurrent downloads share one budget", async () => {
    // The property, not the throughput: two downloads must draw from one pool
    // rather than taking a full one each. Two at 64 is measurably slower than
    // two splitting 64, because throughput turns over past 64.
    const before = await concurrency();
    assert(before.total > 0, "no budget reported");
    assertEquals(before.available, before.total, "budget not idle at rest");

    let lowest = before.total;
    const watch = async () => {
      for (let i = 0; i < 40; i += 1) {
        const now = await concurrency();
        if (now.available < lowest) lowest = now.available;
        assert(
          now.total === before.total,
          `the budget changed mid-download: ${before.total} -> ${now.total}`,
        );
        await new Promise((r) => setTimeout(r, 50));
      }
    };

    const a = install({ app: 896660, dir: scratch("multi-a") });
    const b = install({ app: 896660, dir: scratch("multi-b") });
    await watch();
    a.cancel();
    b.cancel();
    await Promise.allSettled([a, b]);

    assert(
      lowest < before.total,
      "two concurrent downloads never drew on the shared budget",
    );
  });

  await test("cancelling a real install stops it", async () => {
    const job = install({ app: 4020, dir: scratch("cancelled") });
    setTimeout(() => job.cancel(), 1_500);
    let message = "";
    try {
      await job;
    } catch (error) {
      message = error instanceof Error ? error.message : String(error);
    }
    assert(message.includes("cancelled"), `unexpected: ${message}`);
  });
} else {
  console.log("  skip live tests (set TAPLINE_LIVE=1 to run them)");
}

function process_env(key: string): string | undefined {
  // deno-lint-ignore no-explicit-any
  const g = globalThis as any;
  if (g.Deno?.env?.get) return g.Deno.env.get(key) ?? undefined;
  return g.process?.env?.[key];
}

function scratch(name: string): string {
  // Never /tmp: it is tmpfs on the development machine, and a depot test there
  // is that many gigabytes of RAM.
  const home = process_env("HOME") ?? ".";
  return `${home}/.cache/tapline-test/js-${name}`;
}

console.log(`\n${ran - failures}/${ran} passed`);
if (failures > 0) {
  const g = globalThis as { Deno?: { exit(code: number): never }; process?: { exitCode?: number } };
  if (g.Deno) g.Deno.exit(1);
  else if (g.process) g.process.exitCode = 1;
}
