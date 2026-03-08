#!/usr/bin/env node

const fs = require("fs");
const path = require("path");

const [, , url, outputPath] = process.argv;

if (!url || !outputPath) {
  console.error("usage: node wasm_sri_smoke.cjs <url> <output-json>");
  process.exit(1);
}

let playwright;
try {
  playwright = require("playwright");
} catch (error) {
  fs.writeFileSync(
    outputPath,
    JSON.stringify(
      {
        available: false,
        reason: `playwright module unavailable: ${error.message}`,
        browsers: [],
      },
      null,
      2,
    ),
  );
  process.exit(0);
}

async function runBrowser(name, browserType) {
  const result = {
    name,
    available: false,
    launched: false,
    success: false,
    url,
    console: [],
    pageErrors: [],
    requestFailures: [],
    responseStatuses: [],
    serviceWorkerController: false,
    integrityFailures: [],
    wasmFailures: [],
  };

  let browser;
  try {
    browser = await browserType.launch({ headless: true });
    result.available = true;
    result.launched = true;

    const context = await browser.newContext();
    const page = await context.newPage();

    page.on("console", (message) => {
      result.console.push({
        type: message.type(),
        text: message.text(),
      });
    });
    page.on("pageerror", (error) => {
      result.pageErrors.push(String(error));
    });
    page.on("requestfailed", (request) => {
      result.requestFailures.push({
        url: request.url(),
        failure: request.failure(),
      });
    });
    page.on("response", (response) => {
      if (response.status() >= 400) {
        result.responseStatuses.push({
          url: response.url(),
          status: response.status(),
        });
      }
    });

    await page.goto(url, { waitUntil: "networkidle", timeout: 60000 });
    await page.waitForTimeout(3000);

    result.serviceWorkerController = await page.evaluate(
      () => Boolean(navigator.serviceWorker && navigator.serviceWorker.controller),
    );

    const consoleErrors = result.console
      .filter((entry) => entry.type === "error")
      .map((entry) => entry.text);
    const problemTexts = [
      ...consoleErrors,
      ...result.pageErrors,
      ...result.requestFailures.map((entry) => `${entry.url}: ${JSON.stringify(entry.failure)}`),
    ];
    result.integrityFailures = problemTexts.filter(
      (entry) =>
        /subresource integrity|failed integrity|integrity.*(mismatch|error|fail)/i.test(entry) &&
        !/currently ignored for preload destinations/i.test(entry),
    );
    result.wasmFailures = problemTexts.filter((entry) =>
      /wasm|webassembly|module script|import/i.test(entry),
    );
    result.success =
      result.pageErrors.length === 0 &&
      result.requestFailures.length === 0 &&
      result.responseStatuses.length === 0 &&
      result.integrityFailures.length === 0 &&
      result.wasmFailures.length === 0;
  } catch (error) {
    result.error = String(error);
  } finally {
    if (browser) {
      await browser.close();
    }
  }

  return result;
}

async function main() {
  const browserTargets = [
    ["chromium", playwright.chromium],
    ["firefox", playwright.firefox],
    ["webkit", playwright.webkit],
  ];
  const browsers = [];
  for (const [name, browserType] of browserTargets) {
    browsers.push(await runBrowser(name, browserType));
  }

  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(
    outputPath,
    JSON.stringify(
      {
        available: true,
        browsers,
      },
      null,
      2,
    ),
  );
}

main().catch((error) => {
  fs.writeFileSync(
    outputPath,
    JSON.stringify(
      {
        available: false,
        reason: String(error),
        browsers: [],
      },
      null,
      2,
    ),
  );
  process.exit(0);
});
