import { detectRuntime } from "../src/ffi.ts";
import {
  concurrency,
  countWorkshop,
  downloadWorkshopItem,
  install,
  Job,
  plan,
  searchWorkshop,
  version,
  workshop,
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

await test("the chain compiles to the text form the ABI takes", () => {
  const text = workshop(4000, 104691717)
    .gma()
    .only("lua/**")
    .pick("lua/autorun/init.lua")
    .text("zip", "/srv/out.zip");
  assert(
    text === "decode gma\nonly lua/**\npick lua/autorun/init.lua\nzip /srv/out.zip\n",
    `unexpected text form:\n${text}`,
  );
});

await test("a chain step returns a new chain rather than mutating one", () => {
  const base = workshop(4000, 104691717).gma().only("lua/**");
  const a = base.pick("a.lua").text("dir", "/x");
  const b = base.pick("b.lua").text("dir", "/x");
  assert(a.includes("a.lua") && !a.includes("b.lua"), `leaked into a:\n${a}`);
  assert(b.includes("b.lua") && !b.includes("a.lua"), `leaked into b:\n${b}`);
});

await test("a bad sort is refused before the search runs", async () => {
  let message = "";
  try {
    // deno-lint-ignore no-explicit-any
    await searchWorkshop({ app: 4000, sort: "nonsense" as any });
  } catch (error) {
    message = error instanceof Error ? error.message : String(error);
  }
  assert(message.includes("nonsense"), `unexpected: ${message}`);
  assert(message.includes("subscribed"), `should list what works: ${message}`);
});

await test("a trend window on another sort is refused", async () => {
  let message = "";
  try {
    await searchWorkshop({ app: 4000, sort: "vote", days: 7 });
  } catch (error) {
    message = error instanceof Error ? error.message : String(error);
  }
  assert(message.includes("trend"), `unexpected: ${message}`);
});

await test("a text sort without text is refused", async () => {
  let message = "";
  try {
    await searchWorkshop({ app: 4000, sort: "text" });
  } catch (error) {
    message = error instanceof Error ? error.message : String(error);
  }
  assert(message.length > 0, "a text sort with no text was accepted");
});

await test("a bad directive fails before anything downloads", async () => {
  let message = "";
  try {
    await workshop(4000, 104691717).decode("rar").dir(scratch("bad-format"));
  } catch (error) {
    message = error instanceof Error ? error.message : String(error);
  }
  assert(message.includes("rar"), `unexpected: ${message}`);
  assert(message.includes("gma"), `the refusal should list what works: ${message}`);
});

if (process_env("TAPLINE_LIVE") === "1") {
  await test("a count matches the search it stands for", async () => {
    const counted = await countWorkshop({ app: 431960, tags: ["Scene"] });
    const searched = await searchWorkshop({ app: 431960, tags: ["Scene"], limit: 1 });
    assert(counted > 0, "counted nothing");
    assert(
      counted === searched.total,
      `count ${counted} disagrees with search total ${searched.total}`,
    );
  });

  await test("tag groups reach Steam as groups", async () => {
    const grouped = await searchWorkshop({
      app: 431960,
      tagGroups: [["Scene", "Video"], ["Anime"]],
      limit: 1,
    });
    const any = await searchWorkshop({
      app: 431960,
      tags: ["Scene", "Video", "Anime"],
      limit: 1,
    });
    assert(grouped.total > 0, "grouped search matched nothing");
    assert(
      grouped.total < any.total,
      `grouped ${grouped.total} should be fewer than any-of ${any.total}`,
    );
  });

  await test("a filtered chain downloads less than the whole archive", async () => {
    const whole = await workshop(4000, 104691717)
      .gma()
      .dir(scratch("chain-all"));

    const part = await workshop(4000, 104691717)
      .gma()
      .only("lua/**")
      .dir(scratch("chain-lua"));

    assert(whole.entries > part.entries, `no narrowing: ${whole.entries} vs ${part.entries}`);
    assert(
      part.bytesDownloaded < whole.bytesDownloaded,
      `a filter fetched no less: ${part.bytesDownloaded} of ${whole.bytesDownloaded}`,
    );
    console.log(
      `    ${part.entries}/${whole.entries} entries, ` +
        `${part.bytesDownloaded} of ${whole.bytesDownloaded} bytes`,
    );
  });

  await test("a chain writes a zip and reports what went into it", async () => {
    const out = `${scratch("chain-zip")}/out.zip`;
    const report = await workshop(4000, 104691717).gma().only("lua/**").zip(out);
    assert(report.entries > 0, "the zip got no entries");
    assert(report.bytesDownloaded > 0, "nothing was downloaded");
  });

  await test("picking a file that is not there is an error, not an empty result", async () => {
    let message = "";
    try {
      await workshop(4000, 104691717)
        .gma()
        .pick("definitely/not/here.lua")
        .dir(scratch("chain-missing"));
    } catch (error) {
      message = error instanceof Error ? error.message : String(error);
    }
    assert(message.length > 0, "a missing pick succeeded silently");
  });

  await test("a workshop search returns usable results", async () => {
    const page = await searchWorkshop({ app: 4000, limit: 5 });
    assert(page.items.length > 0, "no results");
    assert(page.total > 1000, `too few matches: ${page.total}`);
    assert(page.nextCursor !== null, "a first page should have a next");
    for (const found of page.items) {
      assert(found.title.length > 0, "a result had no title");
      assert(/^\d+$/.test(found.item), `id is not a string of digits: ${found.item}`);
    }
    console.log(`    ${page.items.length} of ${page.total}, first: ${page.items[0]?.title}`);
  });

  await test("a search result downloads without a second lookup", async () => {
    const page = await searchWorkshop({ app: 4000, sort: "subscribed", limit: 20 });
    const small = page.items
      .filter((f) => f.size > 0 && f.size < 8_000_000)
      .sort((a, b) => a.size - b.size)[0];
    assert(small !== undefined, "no small item among the most-subscribed");

    const dir = scratch("js-search-download");
    const report = await downloadWorkshopItem({
      app: 4000,
      item: small.item,
      dir,
      layout: "flat",
    });
    assert(report.files > 0, "nothing was downloaded");
    console.log(`    downloaded ${small.title} (${small.size} bytes)`);
  });

  await test("the search cursor walks forward", async () => {
    const first = await searchWorkshop({ app: 4000, limit: 10 });
    assert(first.nextCursor !== null, "no next cursor");
    const second = await searchWorkshop({ app: 4000, limit: 10, cursor: first.nextCursor });
    const ids = new Set(first.items.map((f) => f.item));
    const overlap = second.items.filter((f) => ids.has(f.item)).length;
    assert(overlap === 0, `the second page repeated ${overlap} items`);
  });

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

  await test("a GMod addon can be downloaded straight into a folder", async () => {
    const dir = scratch("addons");
    const report = await downloadWorkshopItem({
      app: 4000,
      item: 104691717n,
      dir,
      layout: "flat",
    });
    assertEquals(report.files, 1, "an addon should be one file");

    const fs = await import("node:fs");
    const entries = fs.readdirSync(dir);
    assertEquals(entries.length, 1, `expected one file, got ${entries.join(",")}`);
    assert(
      entries[0]?.endsWith(".gma"),
      `expected a .gma, got ${entries[0]}`,
    );
    const head = fs.readFileSync(`${dir}/${entries[0]}`).subarray(0, 4).toString();
    assertEquals(head, "GMAD", "the file is not a Garry's Mod addon");
  });

  await test("an addon can be unpacked and zipped as it lands", async () => {
    const dir = scratch("ext");
    const extended: Record<string, number> = {};
    await downloadWorkshopItem({
      app: 4000,
      item: 104691717n,
      dir,
      layout: "flat",
      extensions: ["gmad", "gmad-zip"],
      onEvent: (event) => {
        if (event.kind === "extended") extended[event.extension] = event.produced;
      },
    });

    const fs = await import("node:fs");
    assert(fs.existsSync(`${dir}/104691717.gma`), "the archive is missing");
    assert(fs.existsSync(`${dir}/104691717.zip`), "the zip was not produced");
    assert(fs.existsSync(`${dir}/104691717`), "the addon was not unpacked");

    assertEquals(extended["gmad"], 348, "wrong unpacked count");
    assertEquals(extended["gmad-zip"], 1, "the zip should be one file");

    const head = fs.readFileSync(`${dir}/104691717.zip`).subarray(0, 2).toString();
    assertEquals(head, "PK", "the zip has no PK signature");
  });

  await test("an addon can be unpacked without ever writing the .gma", async () => {
    const dir = scratch("stream");
    const report = await downloadWorkshopItem({
      app: 4000,
      item: 104691717n,
      dir,
      stream: true,
    });

    assertEquals(report.files, 348, "wrong file count");
    assert(report.bytesStreamed > 8_000_000, "streamed too little");
    const fs = await import("node:fs");
    assert(
      !fs.existsSync(`${dir}/104691717.gma`),
      "the .gma was written after all",
    );
    assert(fs.existsSync(`${dir}/lua`), "the addon was not unpacked");
    assert(
      report.peakBufferedChunks <= 16,
      `buffered ${report.peakBufferedChunks} chunks, past the window`,
    );
  });

  await test("an addon can be streamed straight into a zip", async () => {
    const dir = scratch("streamzip");
    const report = await downloadWorkshopItem({
      app: 4000,
      item: 104691717n,
      dir,
      stream: "zip",
    });
    assertEquals(report.files, 348, "wrong entry count");

    const fs = await import("node:fs");
    const entries = fs.readdirSync(dir);
    assertEquals(entries.length, 1, `expected one file, got ${entries.join(",")}`);
    assertEquals(entries[0], "104691717.zip");
    const head = fs.readFileSync(`${dir}/104691717.zip`).subarray(0, 2).toString();
    assertEquals(head, "PK", "not a zip");
  });

  await test("an unknown extension is refused rather than ignored", async () => {
    let message = "";
    try {
      await downloadWorkshopItem({
        app: 4000,
        item: 104691717n,
        dir: scratch("ext-bogus"),
        extensions: ["definitely-not-real"],
      });
    } catch (error) {
      message = error instanceof Error ? error.message : String(error);
    }
    assert(
      message.includes("unknown extension"),
      `expected a refusal, got: ${message}`,
    );
  });

  await test("the default layout is still steamcmd's", async () => {
    const dir = scratch("addons-nested");
    await downloadWorkshopItem({ app: 4000, item: 104691717n, dir });
    const fs = await import("node:fs");
    assert(
      fs.existsSync(`${dir}/steamapps/workshop/content/4000/104691717`),
      "the steamcmd layout moved, which would relocate every existing consumer's files",
    );
  });

  await test("concurrent downloads share one budget", async () => {
    let before = await concurrency();
    for (let i = 0; i < 100 && before.available < before.total; i += 1) {
        await new Promise((resolve) => setTimeout(resolve, 20));
        before = await concurrency();
    }
    assert(before.total > 0, "no budget reported");
    assertEquals(before.available, before.total, "budget never returned to idle");

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
  const home = process_env("HOME") ?? ".";
  const path = `${home}/.cache/tapline-test/js-${name}`;
  try {
    // deno-lint-ignore no-explicit-any
    const fs = (globalThis as any).process
      ? require_fs()
      : undefined;
    fs?.rmSync(path, { recursive: true, force: true });
  } catch {
  }
  return path;
}

// deno-lint-ignore no-explicit-any
function require_fs(): any {
  // deno-lint-ignore no-explicit-any
  const g = globalThis as any;
  return g.process?.getBuiltinModule?.("node:fs");
}

console.log(`\n${ran - failures}/${ran} passed`);
if (failures > 0) {
  const g = globalThis as { Deno?: { exit(code: number): never }; process?: { exitCode?: number } };
  if (g.Deno) g.Deno.exit(1);
  else if (g.process) g.process.exitCode = 1;
}
