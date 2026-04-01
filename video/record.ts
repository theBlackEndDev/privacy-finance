#!/usr/bin/env npx tsx
/**
 * Automated screen recording script for PrivacyFinance demo video.
 *
 * What it does:
 * 1. Launches Xvfb (virtual display) at 1920x1080
 * 2. Starts the app on that display via tauri-driver + WebDriver
 * 3. Starts ffmpeg recording the virtual display
 * 4. Drives the app through each feature scene using WebDriver commands
 * 5. Stops recording, splits into per-feature clips
 * 6. Renders the final video via Remotion
 *
 * Usage: npx tsx video/record.ts
 */

import { spawn, execSync, type ChildProcess } from 'child_process';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { existsSync, mkdirSync, unlinkSync } from 'fs';
import { Socket } from 'net';

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(__dirname, '..');
const videoDir = __dirname;
const publicDir = join(videoDir, 'public');
const recordingsDir = join(publicDir, 'recordings');
const outDir = join(videoDir, 'out');

// ── Configuration ────────────────────────────────────────────────
const DISPLAY = ':99';
const WIDTH = 1920;
const HEIGHT = 1080;
const FPS = 30;
const DRIVER_PORT = 4444;
const isMac = process.platform === 'darwin';

const appBinary = join(projectRoot, 'src-tauri/target/debug/tauri-app');
const driverBin = isMac ? 'tauri-webdriver' : 'tauri-driver';
const driverArgs = isMac ? ['--port', String(DRIVER_PORT)] : [];

// Feature scenes — each gets driven and recorded as a separate clip
const SCENES = [
  { id: 'onboarding',   duration: 7,  action: driveOnboarding },
  { id: 'dashboard',    duration: 8,  action: driveDashboard },
  { id: 'transactions', duration: 8,  action: driveTransactions },
  { id: 'csv-import',   duration: 7,  action: driveImport },
  { id: 'ai-search',    duration: 7,  action: driveAISearch },
  { id: 'budgets',      duration: 6,  action: driveBudgets },
  { id: 'reports',      duration: 6,  action: driveReports },
  { id: 'recurring',    duration: 6,  action: driveRecurring },
  { id: 'rules',        duration: 5,  action: driveRules },
] as const;

// ── Process management ───────────────────────────────────────────
const processes: ChildProcess[] = [];

function cleanup() {
  console.log('\n[cleanup] Stopping all processes...');
  for (const p of processes) {
    try { p.kill('SIGTERM'); } catch {}
  }
  // Kill Xvfb on the display
  try { execSync(`pkill -f "Xvfb ${DISPLAY}"`, { stdio: 'ignore' }); } catch {}
}

process.on('SIGINT', () => { cleanup(); process.exit(1); });
process.on('SIGTERM', () => { cleanup(); process.exit(1); });
process.on('uncaughtException', (e) => { console.error(e); cleanup(); process.exit(1); });

function waitForPort(port: number, timeout = 15000): Promise<void> {
  return new Promise((resolve, reject) => {
    const deadline = Date.now() + timeout;
    const tryConnect = () => {
      if (Date.now() > deadline) return reject(new Error(`Port ${port} not ready`));
      const socket = new Socket();
      socket.setTimeout(500);
      socket.once('connect', () => { socket.destroy(); resolve(); });
      socket.once('timeout', () => { socket.destroy(); setTimeout(tryConnect, 200); });
      socket.once('error', () => { setTimeout(tryConnect, 200); });
      socket.connect(port, '127.0.0.1');
    };
    tryConnect();
  });
}

function sleep(ms: number): Promise<void> {
  return new Promise(r => setTimeout(r, ms));
}

// ── WebDriver helpers ────────────────────────────────────────────
const WD_URL = `http://127.0.0.1:${DRIVER_PORT}`;
let sessionId = '';

async function wdRequest(method: string, path: string, body?: unknown) {
  const url = `${WD_URL}${path}`;
  const res = await fetch(url, {
    method,
    headers: { 'Content-Type': 'application/json' },
    body: body ? JSON.stringify(body) : undefined,
  });
  const json = await res.json();
  if (json.value?.error) throw new Error(`WebDriver: ${json.value.message}`);
  return json.value;
}

async function createSession(): Promise<string> {
  const result = await wdRequest('POST', '/session', {
    capabilities: {
      alwaysMatch: {
        'tauri:options': { application: appBinary },
      },
    },
  });
  return result.sessionId;
}

async function deleteSession() {
  if (sessionId) {
    await wdRequest('DELETE', `/session/${sessionId}`).catch(() => {});
    sessionId = '';
  }
}

/** Execute JavaScript in the app */
async function executeJS(script: string, args: unknown[] = []): Promise<unknown> {
  return wdRequest('POST', `/session/${sessionId}/execute/sync`, { script, args });
}

/** Wait for an element to exist and be visible */
async function waitFor(selector: string, timeout = 10000): Promise<void> {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    const visible = await executeJS(`
      const el = document.querySelector('${selector}');
      return el ? el.offsetParent !== null || el.style.display !== 'none' : false;
    `);
    if (visible) return;
    await sleep(250);
  }
  throw new Error(`Timeout waiting for: ${selector}`);
}

/** Click an element via JS */
async function click(selector: string) {
  await executeJS(`document.querySelector('${selector}')?.click()`);
}

/** Set input value via React-compatible method */
async function setInput(selector: string, value: string) {
  await executeJS(`
    const el = document.querySelector('${selector}');
    if (!el) return;
    const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set;
    if (setter) {
      setter.call(el, '${value}');
      el.dispatchEvent(new Event('input', { bubbles: true }));
      el.dispatchEvent(new Event('change', { bubbles: true }));
    }
  `);
}

/** Select dropdown option by text */
async function selectByText(selector: string, text: string) {
  await executeJS(`
    const el = document.querySelector('${selector}');
    if (!el) return;
    const opt = Array.from(el.options).find(o => o.text.includes('${text}'));
    if (opt) {
      el.value = opt.value;
      el.dispatchEvent(new Event('change', { bubbles: true }));
    }
  `);
}

/** Navigate to a view via sidebar */
async function navigateTo(testId: string) {
  await click(`[data-testid="nav-${testId}"]`);
  await sleep(800);
}

// ── ffmpeg recording ─────────────────────────────────────────────

function startRecording(outputPath: string): ChildProcess {
  const args = [
    '-y',
    '-video_size', `${WIDTH}x${HEIGHT}`,
    '-framerate', String(FPS),
    '-f', 'x11grab',
    '-i', DISPLAY,
    '-c:v', 'libx264',
    '-preset', 'ultrafast',
    '-crf', '18',
    '-pix_fmt', 'yuv420p',
    '-movflags', '+faststart',
    outputPath,
  ];
  const proc = spawn('ffmpeg', args, { stdio: ['pipe', 'ignore', 'ignore'] });
  processes.push(proc);
  return proc;
}

function stopRecording(proc: ChildProcess): Promise<void> {
  return new Promise((resolve) => {
    proc.on('close', () => resolve());
    // 'q' to stdin is the cleanest way to finalize the MP4 moov atom
    proc.stdin?.write('q');
    proc.stdin?.end();
    // Fallback: SIGINT after 5s (ffmpeg handles it gracefully, unlike SIGTERM/SIGKILL)
    setTimeout(() => { try { proc.kill('SIGINT'); } catch {} }, 5000);
  });
}

/** Trim a video clip using ffmpeg */
function trimClip(input: string, output: string, startSec: number, durationSec: number) {
  execSync(
    `ffmpeg -y -ss ${startSec} -t ${durationSec} -i "${input}" -c copy "${output}"`,
    { stdio: 'ignore' }
  );
}

// ── Feature scene drivers ────────────────────────────────────────
// Each function drives the app through one feature, creating a natural
// screen recording that shows the feature in action.

async function driveOnboarding() {
  // Fresh DB — wizard should be showing
  await waitFor('[data-testid="onboarding-wizard"]', 15000);
  await sleep(1000);

  // Click "Get started"
  await click('[data-testid="onboarding-next"]');
  await sleep(1000);

  // Fill account name and balance
  await setInput('[data-testid="onboarding-account-name"]', 'Primary Checking');
  await sleep(500);
  await setInput('[data-testid="onboarding-balance"]', '5000');
  await sleep(500);

  // Submit the form
  await executeJS(`
    const btn = document.querySelector('[data-testid="onboarding-next"]');
    if (btn) {
      const form = btn.closest('form');
      if (form) form.requestSubmit(btn);
      else btn.click();
    }
  `);
  await sleep(2000);

  // Click "Start using PrivacyFinance"
  await waitFor('[data-testid="onboarding-finish"]', 10000);
  await sleep(500);
  await click('[data-testid="onboarding-finish"]');
  await sleep(1500);

  // Dismiss any modal
  const hasModal = await executeJS(`
    const el = document.querySelector('[data-testid="modal-close"]');
    return el ? el.offsetParent !== null : false;
  `);
  if (hasModal) {
    await click('[data-testid="modal-close"]');
    await sleep(500);
  }
}

async function driveDashboard() {
  await navigateTo('dashboard');
  await sleep(2000);
  // Just let the dashboard render — the visual speaks for itself
  await sleep(3000);
}

async function driveTransactions() {
  await navigateTo('transactions');
  await sleep(1000);

  // Add an expense
  await click('[data-testid="tx-kind-select"] button');
  await sleep(300);
  await setInput('[data-testid="tx-payee-input"]', 'Whole Foods');
  await sleep(400);
  await setInput('[data-testid="tx-amount-input"]', '85.50');
  await sleep(400);
  await selectByText('[data-testid="tx-category-select"]', 'Food');
  await sleep(300);
  await click('[data-testid="tx-submit-btn"]');
  await sleep(1000);

  // Add another one for visual variety
  await setInput('[data-testid="tx-payee-input"]', 'Netflix');
  await sleep(400);
  await setInput('[data-testid="tx-amount-input"]', '15.99');
  await sleep(400);
  await selectByText('[data-testid="tx-category-select"]', 'Subscriptions');
  await sleep(300);
  await click('[data-testid="tx-submit-btn"]');
  await sleep(1500);
}

async function driveImport() {
  await navigateTo('import');
  await sleep(2000);
  // Show the import page — we can't fully drive a file picker in headless,
  // but showing the UI is enough for the demo
  await sleep(3000);
}

async function driveAISearch() {
  // Click on the search input in the header
  await click('[data-testid="search-input"]');
  await sleep(500);

  // Type a natural language query slowly for visual effect
  const query = 'How much did I spend on food?';
  for (const char of query) {
    await executeJS(`
      const el = document.querySelector('[data-testid="search-input"]');
      if (el) {
        const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set;
        if (setter) {
          setter.call(el, el.value + '${char}');
          el.dispatchEvent(new Event('input', { bubbles: true }));
        }
      }
    `);
    await sleep(60);
  }

  // Wait for AI card to appear
  await sleep(2000);
  await waitFor('[data-testid="ai-search-card"]', 10000);
  await sleep(2000);

  // Clear search
  await setInput('[data-testid="search-input"]', '');
  await sleep(500);
}

async function driveBudgets() {
  await navigateTo('budgets');
  await sleep(1500);

  // Set a budget amount for the first category
  const hasInput = await executeJS(`
    const el = document.querySelector('[data-testid^="budget-amount-"]');
    return !!el;
  `);
  if (hasInput) {
    await executeJS(`
      const el = document.querySelector('[data-testid^="budget-amount-"]');
      if (el) {
        el.click();
        const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set;
        if (setter) {
          setter.call(el, '300');
          el.dispatchEvent(new Event('input', { bubbles: true }));
          el.dispatchEvent(new Event('change', { bubbles: true }));
        }
      }
    `);
    await sleep(500);
    // Click elsewhere to save
    await click('[data-testid="budget-month-nav"]');
  }
  await sleep(2500);
}

async function driveReports() {
  await navigateTo('reports');
  await sleep(1500);

  // Click through report tabs
  await click('[data-testid="report-income-expense"]');
  await sleep(1500);
  await click('[data-testid="report-category-spending"]');
  await sleep(1500);
  await click('[data-testid="report-net-worth"]');
  await sleep(1500);
}

async function driveRecurring() {
  await navigateTo('recurring');
  await sleep(1000);

  // Create a recurring transaction
  await click('[data-testid="recurring-create-btn"]');
  await sleep(500);
  await setInput('[data-testid="recurring-payee-input"]', 'Rent');
  await sleep(400);
  await setInput('[data-testid="recurring-amount-input"]', '1850');
  await sleep(400);
  await selectByText('[data-testid="recurring-freq-select"]', 'Monthly');
  await sleep(300);
  await click('[data-testid="recurring-submit-btn"]');
  await sleep(1500);
}

async function driveRules() {
  await navigateTo('rules');
  await sleep(1000);

  // Create an auto-categorization rule
  await click('[data-testid="rule-create-btn"]');
  await sleep(500);
  await setInput('[data-testid="rule-name-input"]', 'Food stores');
  await sleep(400);
  await setInput('[data-testid="rule-pattern-input"]', 'Whole Foods');
  await sleep(400);
  await selectByText('[data-testid="rule-category-select"]', 'Food');
  await sleep(300);
  await click('[data-testid="rule-submit-btn"]');
  await sleep(1500);
}

// ── Main pipeline ────────────────────────────────────────────────

async function main() {
  console.log('╔══════════════════════════════════════════════════════════╗');
  console.log('║  PrivacyFinance Demo Video — Automated Recording        ║');
  console.log('╚══════════════════════════════════════════════════════════╝\n');

  // Ensure directories exist
  mkdirSync(recordingsDir, { recursive: true });
  mkdirSync(outDir, { recursive: true });

  // ── Step 0: Build the app (if binary missing) ────────────────
  if (!existsSync(appBinary)) {
    console.log('[0/6] Building app binary (this takes a few minutes)...');
    execSync('npm run test:e2e:build', { cwd: projectRoot, stdio: 'inherit' });
    console.log('    ✓ App binary built\n');
  } else {
    console.log('[0/6] App binary found, skipping build\n');
  }

  // ── Step 1: Launch Xvfb ──────────────────────────────────────
  if (!isMac) {
    console.log('[1/6] Starting Xvfb virtual display...');
    try { execSync(`pkill -f "Xvfb ${DISPLAY}"`, { stdio: 'ignore' }); } catch {}
    const xvfb = spawn('Xvfb', [DISPLAY, '-screen', '0', `${WIDTH}x${HEIGHT}x24`], {
      stdio: 'ignore',
    });
    processes.push(xvfb);
    await sleep(1000);
    process.env.DISPLAY = DISPLAY;
    console.log(`    ✓ Xvfb running on ${DISPLAY} at ${WIDTH}x${HEIGHT}\n`);
  } else {
    console.log('[1/6] macOS — using native display\n');
  }

  // ── Step 2: Reset DB and start driver ────────────────────────
  console.log('[2/6] Resetting database and starting WebDriver...');
  // Reset DB
  const { resetDatabase } = await import('../e2e/helpers/app.js');
  resetDatabase();

  const driver = spawn(driverBin, driverArgs, {
    stdio: ['ignore', 'ignore', 'ignore'],
    env: { ...process.env, DISPLAY },
  });
  processes.push(driver);
  await waitForPort(DRIVER_PORT);
  console.log(`    ✓ ${driverBin} ready on port ${DRIVER_PORT}\n`);

  // ── Step 3: Create WebDriver session ─────────────────────────
  console.log('[3/6] Creating WebDriver session...');
  sessionId = await createSession();
  console.log(`    ✓ Session: ${sessionId}\n`);

  // Give the app time to start
  await sleep(3000);

  // ── Step 4: Record each feature scene ────────────────────────
  console.log('[4/6] Recording feature scenes...\n');

  for (const scene of SCENES) {
    const clipPath = join(recordingsDir, `${scene.id}.mp4`);
    console.log(`  ▶ Recording: ${scene.id} (${scene.duration}s)...`);

    // Start recording
    const recorder = startRecording(clipPath);
    await sleep(500); // Let ffmpeg initialize

    try {
      // Drive the feature
      await scene.action();
      // Hold for remaining duration
      await sleep(Math.max(0, scene.duration * 1000 - 2000));
    } catch (e) {
      console.error(`    ✗ Error in ${scene.id}:`, e);
    }

    // Stop recording
    await stopRecording(recorder);
    console.log(`    ✓ Saved: ${scene.id}.mp4`);
  }
  console.log();

  // ── Step 5: Clean up WebDriver ───────────────────────────────
  console.log('[5/6] Closing WebDriver session...');
  await deleteSession();
  console.log('    ✓ Session closed\n');

  // ── Step 6: Render with Remotion ─────────────────────────────
  console.log('[6/6] Rendering final video with Remotion...');

  // Update FeatureScene to use actual recordings
  // The FeatureScene component checks for files in public/recordings/

  try {
    execSync(
      `npx remotion render src/index.ts PrivacyFinanceDemo out/demo.mp4 --log=error`,
      { cwd: videoDir, stdio: 'inherit', env: { ...process.env, DISPLAY } }
    );
    console.log(`\n    ✓ Final video: ${join(outDir, 'demo.mp4')}\n`);
  } catch (e) {
    console.error('    ✗ Remotion render failed:', e);
    console.log('    Clips are still saved in public/recordings/ for manual composition.\n');
  }

  cleanup();
  console.log('Done!');
}

main().catch((e) => {
  console.error('Fatal:', e);
  cleanup();
  process.exit(1);
});
