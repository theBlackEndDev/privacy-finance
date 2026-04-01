import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { createRequire } from 'module';
import { spawn, type ChildProcess } from 'child_process';
import { resetDatabase } from './helpers/app.js';

const require = createRequire(import.meta.url);
const { waitTauriDriverReady } = require('@crabnebula/tauri-driver/main.js');

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(__dirname, '..');

const isMac = process.platform === 'darwin';
const appBinary = isMac
  ? join(projectRoot, 'src-tauri/target/release/bundle/macos/PrivacyFinance.app')
  : join(projectRoot, 'src-tauri/target/release/tauri-app');

let tauriDriver: ChildProcess | null = null;
let driverKilled = false;

export const config = {
  host: '127.0.0.1',
  port: 4444,

  specs: [join(__dirname, 'specs/**/*.spec.ts')],
  maxInstances: 1,
  connectionRetryCount: 0,

  capabilities: [{
    maxInstances: 1,
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
    const driverBin = join(projectRoot, 'node_modules', '.bin', 'tauri-driver');
    tauriDriver = spawn(driverBin, [], {
      stdio: [null, process.stdout, process.stderr],
    });

    tauriDriver.on('error', (err: Error) => {
      console.error('[E2E] tauri-driver error:', err);
    });
    tauriDriver.on('exit', (code: number | null) => {
      if (!driverKilled) {
        console.error('[E2E] tauri-driver exited unexpectedly with code:', code);
      }
    });

    await waitTauriDriverReady('127.0.0.1', 4444, 200);
  },

  afterSession() {
    driverKilled = true;
    tauriDriver?.kill();
    tauriDriver = null;
  },
};
