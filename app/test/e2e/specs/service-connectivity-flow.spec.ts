import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';

import { waitForApp, waitForAppReady } from '../helpers/app-helpers';
import { triggerAuthDeepLink } from '../helpers/deep-link-helpers';
import {
  clickButton,
  textExists,
  waitForText,
  waitForWebView,
  waitForWindowVisible,
} from '../helpers/element-helpers';
import { startMockServer, stopMockServer } from '../mock-server';

interface ServiceMockFailures {
  install?: string;
  start?: string;
  stop?: string;
  status?: string;
  uninstall?: string;
}

interface ServiceMockState {
  installed: boolean;
  running: boolean;
  agent_running: boolean;
  failures: ServiceMockFailures;
}

const DEFAULT_MOCK_STATE: ServiceMockState = {
  installed: false,
  running: false,
  agent_running: false,
  failures: {},
};

const mockStateFile =
  process.env.OPENHUMAN_SERVICE_MOCK_STATE_FILE ||
  path.join(process.env.OPENHUMAN_WORKSPACE || os.tmpdir(), 'service-mock-state.json');

async function writeMockState(state: ServiceMockState): Promise<void> {
  await fs.mkdir(path.dirname(mockStateFile), { recursive: true });
  await fs.writeFile(mockStateFile, JSON.stringify(state, null, 2), 'utf-8');
}

async function readMockState(): Promise<ServiceMockState> {
  const raw = await fs.readFile(mockStateFile, 'utf-8');
  return JSON.parse(raw) as ServiceMockState;
}

async function waitForServiceStateText(stateText: string, timeoutMs = 15_000): Promise<void> {
  await waitForText(stateText, timeoutMs);
}

describe('Service connectivity flow (UI ↔ Rust service)', () => {
  before(async function beforeSuite() {
    if (process.env.OPENHUMAN_SERVICE_MOCK !== '1') {
      this.skip();
    }

    await writeMockState(DEFAULT_MOCK_STATE);
    await startMockServer();
    await waitForApp();

    await triggerAuthDeepLink('service-connectivity-token');
    await waitForWindowVisible(25_000);
    await waitForWebView(15_000);
    await waitForAppReady(15_000);
  });

  after(async () => {
    await stopMockServer();
  });

  it('shows the blocking gate when service is not installed', async () => {
    await waitForText('OpenHuman Service Required', 20_000);
    await waitForServiceStateText('NotInstalled');

    expect(await textExists('Install Service')).toBe(true);
    expect(await textExists('Start Service')).toBe(true);
    expect(await textExists('Stop Service')).toBe(true);
    expect(await textExists('Restart Service')).toBe(true);
    expect(await textExists('Uninstall Service')).toBe(true);
  });

  it('installs the service from the gate', async () => {
    await clickButton('Install Service');
    await waitForServiceStateText('Stopped');

    const state = await readMockState();
    expect(state.installed).toBe(true);
    expect(state.running).toBe(false);
  });

  it('starts the service from the gate', async () => {
    await clickButton('Start Service');
    await waitForServiceStateText('Running');

    const state = await readMockState();
    expect(state.installed).toBe(true);
    expect(state.running).toBe(true);
  });

  it('stops the service from the gate', async () => {
    await clickButton('Stop Service');
    await waitForServiceStateText('Stopped');

    const state = await readMockState();
    expect(state.running).toBe(false);
  });

  it('restarts the service from the gate', async () => {
    await clickButton('Restart Service');
    await waitForServiceStateText('Running');

    const state = await readMockState();
    expect(state.running).toBe(true);
  });

  it('keeps user blocked and surfaces error when core start fails', async () => {
    const state = await readMockState();
    await writeMockState({
      ...state,
      running: false,
      failures: { ...state.failures, start: 'simulated start failure' },
    });

    await clickButton('Refresh');
    await waitForServiceStateText('Stopped');

    await clickButton('Start Service');
    await waitForText('simulated start failure', 10_000);
    await waitForText('OpenHuman Service Required', 10_000);

    const latest = await readMockState();
    expect(latest.running).toBe(false);
  });

  it('uninstalls the service from the gate', async () => {
    const state = await readMockState();
    await writeMockState({
      ...state,
      failures: { ...state.failures, start: undefined },
    });

    await clickButton('Uninstall Service');
    await waitForServiceStateText('NotInstalled');

    const latest = await readMockState();
    expect(latest.installed).toBe(false);
    expect(latest.running).toBe(false);
  });

  it('unblocks once service is running and agent is reported healthy', async () => {
    const state = await readMockState();
    await writeMockState({
      ...state,
      installed: true,
      running: true,
      agent_running: true,
      failures: {},
    });

    await clickButton('Refresh');

    await browser.waitUntil(async () => !(await textExists('OpenHuman Service Required')), {
      timeout: 20_000,
      timeoutMsg: 'Service blocking gate did not clear after healthy status',
    });
  });
});
