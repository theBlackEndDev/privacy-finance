import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { spawn, type ChildProcess } from 'child_process';
import { Socket } from 'net';
import { resetDatabase } from './helpers/app.js';

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(__dirname, '..');
const isMac = process.platform === 'darwin';

// tauri build --debug --no-bundle produces the binary here
const appBinary = join(projectRoot, 'src-tauri/target/debug/tauri-app');

// Linux: official tauri-driver (uses WebKitWebDriver under the hood)
// macOS: tauri-webdriver (uses tauri-plugin-webdriver embedded in app)
const driverBin = isMac ? 'tauri-webdriver' : 'tauri-driver';
const driverArgs = isMac ? ['--port', '4444'] : [];

let driverProcess: ChildProcess | null = null;
let driverKilled = false;

function waitForPort(host: string, port: number, interval = 200): Promise<void> {
  return new Promise((resolve) => {
    const tryConnect = () => {
      const socket = new Socket();
      socket.setTimeout(1000);
      socket.once('connect', () => { socket.destroy(); resolve(); });
      socket.once('timeout', () => { socket.destroy(); setTimeout(tryConnect, interval); });
      socket.once('error', () => { setTimeout(tryConnect, interval); });
      socket.connect(port, host);
    };
    tryConnect();
  });
}

export const config = {
  hostname: 'localhost',
  port: 4444,
  path: '/',

  specs: [join(__dirname, 'specs/**/*.spec.ts')],
  maxInstances: 1,
  connectionRetryCount: 0,

  capabilities: [{
    'tauri:options': {
      application: appBinary,
    },
  }],

  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: {
    ui: 'bdd',
    timeout: 60000,
  },

  autoCompileOpts: {
    tsNodeOpts: {
      project: join(__dirname, 'tsconfig.json'),
    },
  },

  async onPrepare() {
    resetDatabase();

    driverProcess = spawn(driverBin, driverArgs, {
      stdio: [null, process.stdout, process.stderr],
    });

    driverProcess.on('error', (err: Error) => {
      console.error(`[E2E] ${driverBin} error:`, err.message);
    });
    driverProcess.on('exit', (code: number | null) => {
      if (!driverKilled) {
        console.error(`[E2E] ${driverBin} exited unexpectedly with code:`, code);
      }
    });

    await waitForPort('127.0.0.1', 4444);
  },

  onComplete() {
    driverKilled = true;
    driverProcess?.kill();
    driverProcess = null;
  },
};
