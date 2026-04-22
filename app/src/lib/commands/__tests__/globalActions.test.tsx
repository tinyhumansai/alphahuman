import { describe, it, expect, beforeEach, vi } from 'vitest';
import { registerGlobalActions, GROUP_ORDER } from '../globalActions';
import { hotkeyManager } from '../hotkeyManager';
import { registry } from '../registry';
import type { NavigateFunction } from 'react-router-dom';

beforeEach(() => {
  hotkeyManager.teardown();
  hotkeyManager.init();
});

describe('registerGlobalActions', () => {
  it('registers the 5 seed nav actions into the global frame', () => {
    const frame = hotkeyManager.pushFrame('global', 'root');
    const navigate = vi.fn() as unknown as NavigateFunction;
    const openHelp = vi.fn();
    registerGlobalActions(navigate, openHelp, frame);
    const ids = ['nav.home', 'nav.chat', 'nav.intelligence', 'nav.skills', 'nav.settings'];
    for (const id of ids) expect(registry.getAction(id)?.id).toBe(id);
    // help action intentionally disabled — re-enable when help overlay returns.
    expect(registry.getAction('help.show')).toBeUndefined();
  });

  it('nav.home handler calls navigate("/home")', () => {
    const frame = hotkeyManager.pushFrame('global', 'root');
    const navigate = vi.fn();
    registerGlobalActions(navigate as unknown as NavigateFunction, vi.fn(), frame);
    registry.setActiveStack([frame]);
    registry.runAction('nav.home');
    expect(navigate).toHaveBeenCalledWith('/home');
  });

  it('exports GROUP_ORDER', () => {
    expect(GROUP_ORDER).toEqual(['Navigation']);
  });
});
