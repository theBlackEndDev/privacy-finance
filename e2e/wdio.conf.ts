import { join } from 'path';
import { resetDatabase } from './helpers/app';

const isMac = process.platform === 'darwin';

// App binary path differs per platform
const appBinary = isMac
  ? join(__dirname, '../src-tauri/target/release/bundle/macos/PrivacyFinance.app')
  : join(__dirname, '../src-tauri/target/release/tauri-app');

export const config = {
  runner: 'local',
  specs: ['./e2e/specs/**/*.spec.ts'],
  maxInstances: 1,
  capabilities: [{
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
  /**
   * Reset the database before the test suite starts to ensure fresh state.
   */
  onPrepare() {
    resetDatabase();
  },
} as const;
