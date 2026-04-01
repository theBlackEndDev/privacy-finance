import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { resetDatabase } from './helpers/app.js';

const __dirname = dirname(fileURLToPath(import.meta.url));
const isMac = process.platform === 'darwin';

const appBinary = isMac
  ? join(__dirname, '../src-tauri/target/release/bundle/macos/PrivacyFinance.app')
  : join(__dirname, '../src-tauri/target/release/tauri-app');

export const config = {
  runner: 'local',
  specs: [join(__dirname, 'specs/**/*.spec.ts')],
  maxInstances: 1,
  capabilities: [{
    browserName: 'tauri',
    'tauri:options': {
      application: appBinary,
    },
  }],
  services: ['@crabnebula/tauri-driver'],
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
} as const;
