import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import CommandProvider from '../CommandProvider';
import { hotkeyManager } from '../../../lib/commands/hotkeyManager';
import { pressKey } from '../../../test/commandTestUtils';

beforeEach(() => {
  hotkeyManager.teardown();
});

describe('CommandProvider', () => {
  it('mounts and registers seed actions', () => {
    render(
      <MemoryRouter>
        <CommandProvider>
          <div>child</div>
        </CommandProvider>
      </MemoryRouter>,
    );
    expect(screen.getByText('child')).toBeInTheDocument();
  });

  it('opens palette on mod+K', async () => {
    render(
      <MemoryRouter>
        <CommandProvider>
          <div>child</div>
        </CommandProvider>
      </MemoryRouter>,
    );
    act(() => {
      pressKey({ key: 'k', mod: true });
    });
    expect(
      await screen.findByRole('dialog', { name: /Command palette/i }),
    ).toBeInTheDocument();
  });

  it('opens help on ?', async () => {
    render(
      <MemoryRouter>
        <CommandProvider>
          <div>child</div>
        </CommandProvider>
      </MemoryRouter>,
    );
    act(() => {
      pressKey({ key: '?' });
    });
    expect(
      await screen.findByRole('dialog', { name: /Keyboard shortcuts/i }),
    ).toBeInTheDocument();
  });

  it('Esc closes open overlay', async () => {
    const user = userEvent.setup();
    render(
      <MemoryRouter>
        <CommandProvider>
          <div>child</div>
        </CommandProvider>
      </MemoryRouter>,
    );
    act(() => {
      pressKey({ key: 'k', mod: true });
    });
    expect(await screen.findByRole('dialog')).toBeInTheDocument();
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('palette and help mutually exclusive (opening help closes palette)', async () => {
    render(
      <MemoryRouter>
        <CommandProvider>
          <div>child</div>
        </CommandProvider>
      </MemoryRouter>,
    );
    act(() => {
      pressKey({ key: 'k', mod: true });
    });
    expect(
      await screen.findByRole('dialog', { name: /Command palette/i }),
    ).toBeInTheDocument();
    act(() => {
      pressKey({ key: '?' });
    });
    expect(
      await screen.findByRole('dialog', { name: /Keyboard shortcuts/i }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole('dialog', { name: /Command palette/i }),
    ).not.toBeInTheDocument();
  });
});
