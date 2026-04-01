import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { spawn, type ChildProcess } from 'child_process';
import { Socket } from 'net';
import { resetDatabase } from './helpers/app.js';

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(__dirname, '..');
const isMac = process.platform === 'darwin';

// Per tauri-webdriver docs: point to the built binary
const appBinary = isMac
  ? join(projectRoot, 'src-tauri/target/release/bundle/macos/PrivacyFinance.app/Contents/MacOS/PrivacyFinance')
  : join(projectRoot, 'src-tauri/target/release/tauri-app');

let driverProcess: ChildProcess | null = null;
let driverKilled = false;

/** Poll a TCP port until something is listening. */
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

  onPrepare() {
    resetDatabase();
  },

  async beforeSession() {
    // tauri-webdriver is installed via: cargo install tauri-webdriver --locked
    driverProcess = spawn('tauri-webdriver', ['--port', '4444'], {
      stdio: [null, process.stdout, process.stderr],
    });

    driverProcess.on('error', (err: Error) => {
      console.error('[E2E] tauri-webdriver error:', err.message);
      console.error('[E2E] Install it with: cargo install tauri-webdriver --locked');
    });
    driverProcess.on('exit', (code: number | null) => {
      if (!driverKilled) {
        console.error('[E2E] tauri-webdriver exited unexpectedly with code:', code);
      }
    });

    // Wait for the WebDriver server to be listening
    await waitForPort('127.0.0.1', 4444);
  },

  afterSession() {
    driverKilled = true;
    driverProcess?.kill();
    driverProcess = null;
  },
};
