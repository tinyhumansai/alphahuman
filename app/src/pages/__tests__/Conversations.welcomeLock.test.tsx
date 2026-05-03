/**
 * Tests for the welcome-lockdown features added in PR #1116:
 *  - filteredThreads: during lockdown only the welcome thread (or onboarding-
 *    labelled threads) appear in the sidebar
 *  - resolveThreadDisplayTitle: returns "Onboarding" for the welcome thread
 *    while locked, falls back to server title otherwise
 *  - effectiveShowSidebar: sidebar is clamped to open during lockdown
 *  - delete button hidden for welcome thread during lockdown
 *  - "New thread" button hidden during lockdown
 *  - Tab-bar label filter hidden during lockdown
 */
import { configureStore } from '@reduxjs/toolkit';
import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Provider } from 'react-redux';
import { MemoryRouter } from 'react-router-dom';

import { ONBOARDING_WELCOME_THREAD_LABEL } from '../../constants/onboardingChat';
import chatRuntimeReducer from '../../store/chatRuntimeSlice';
import socketReducer from '../../store/socketSlice';
import threadReducer from '../../store/threadSlice';
import type { Thread } from '../../types/thread';

// ── Module-level mocks ─────────────────────────────────────────────────────

vi.mock('../../providers/CoreStateProvider', () => ({
  useCoreState: vi.fn(),
}));

vi.mock('../../lib/coreState/store', () => ({
  isWelcomeLocked: vi.fn(),
  getCoreStateSnapshot: vi.fn(),
}));

vi.mock('../../services/chatService', () => ({
  chatSend: vi.fn(),
  chatCancel: vi.fn(),
  useRustChat: vi.fn(() => true),
}));

vi.mock('../../hooks/useUsageState', () => ({
  useUsageState: () => ({
    teamUsage: null,
    currentPlan: null,
    currentTier: 'free',
    isFreeTier: true,
    usagePct10h: 0,
    usagePct7d: 0,
    isNearLimit: false,
    isAtLimit: false,
    isRateLimited: false,
    isBudgetExhausted: false,
    shouldShowBudgetCompletedMessage: false,
    isLoading: false,
    refresh: vi.fn(),
  }),
}));

vi.mock('../../hooks/useStickToBottom', () => ({
  useStickToBottom: () => ({
    containerRef: { current: null },
    endRef: { current: null },
  }),
}));

vi.mock('../../components/chat/TokenUsagePill', () => ({
  default: () => <span data-testid="token-usage-pill" />,
}));

vi.mock('../../components/intelligence/ConfirmationModal', () => ({
  ConfirmationModal: () => null,
}));

vi.mock('../../components/PillTabBar', () => ({
  default: ({ items }: { items: { label: string; value: string }[] }) => (
    <div data-testid="pill-tab-bar">
      {items.map(i => (
        <span key={i.value}>{i.label}</span>
      ))}
    </div>
  ),
}));

vi.mock('../../components/upsell/UpsellBanner', () => ({
  default: () => null,
}));

vi.mock('../../components/upsell/UsageLimitModal', () => ({
  default: () => null,
}));

vi.mock('../../components/upsell/upsellDismissState', () => ({
  shouldShowBanner: vi.fn(() => false),
  dismissBanner: vi.fn(),
}));

vi.mock('../../utils/openUrl', () => ({ openUrl: vi.fn() }));

vi.mock('./conversations/components/AgentMessageBubble', () => ({
  AgentMessageBubble: () => null,
  BubbleMarkdown: () => null,
}));

vi.mock('./conversations/components/CitationChips', () => ({
  CitationChips: () => null,
}));

vi.mock('./conversations/components/LimitPill', () => ({
  LimitPill: () => null,
}));

vi.mock('./conversations/components/ToolTimelineBlock', () => ({
  ToolTimelineBlock: () => null,
}));

vi.mock('./conversations/utils/format', () => ({
  buildAcceptedInlineCompletion: vi.fn(() => ''),
  formatRelativeTime: vi.fn(() => ''),
  formatResetTime: vi.fn(() => ''),
  getInlineCompletionSuffix: vi.fn(() => ''),
}));

// Mock the async thunks so they don't make real API calls.
// We return no-op thunk functions that resolve immediately so the
// component's useEffect can complete without errors.
vi.mock('../../services/api/threadApi', () => ({
  threadApi: {
    createNewThread: vi.fn().mockResolvedValue({ id: 'new-t', labels: [] }),
    getThreads: vi.fn().mockResolvedValue({ threads: [], count: 0 }),
    getThreadMessages: vi.fn().mockResolvedValue({ messages: [], count: 0 }),
    appendMessage: vi.fn().mockResolvedValue({}),
    deleteThread: vi.fn().mockResolvedValue({ deleted: true }),
    generateTitleIfNeeded: vi.fn().mockResolvedValue({}),
    updateMessage: vi.fn().mockResolvedValue({}),
    purge: vi.fn().mockResolvedValue({}),
    updateLabels: vi.fn().mockResolvedValue({}),
  },
}));

// ── Helpers ────────────────────────────────────────────────────────────────

function makeThread(overrides: Partial<Thread> = {}): Thread {
  return {
    id: 'thread-1',
    title: 'My Thread',
    chatId: null,
    isActive: false,
    messageCount: 0,
    lastMessageAt: '2026-01-01T00:00:00.000Z',
    createdAt: '2026-01-01T00:00:00.000Z',
    labels: [],
    ...overrides,
  };
}

function buildStore(overrides: {
  threads?: Thread[];
  selectedThreadId?: string | null;
  welcomeThreadId?: string | null;
}) {
  const { threads = [], selectedThreadId = null, welcomeThreadId = null } = overrides;

  return configureStore({
    reducer: {
      thread: threadReducer,
      chatRuntime: chatRuntimeReducer,
      socket: socketReducer,
    },
    preloadedState: {
      thread: {
        threads,
        selectedThreadId,
        welcomeThreadId,
        activeThreadId: null,
        messagesByThreadId: {},
        messages: [],
        isLoadingThreads: false,
        isLoadingMessages: false,
        messagesError: null,
      },
    } as never,
  });
}

async function renderConversations(opts: {
  welcomeLocked: boolean;
  threads?: Thread[];
  selectedThreadId?: string | null;
  welcomeThreadId?: string | null;
}) {
  const {
    welcomeLocked,
    threads = [],
    selectedThreadId = null,
    welcomeThreadId = null,
  } = opts;

  const { useCoreState } = await import('../../providers/CoreStateProvider');
  const coreStateModule = await import('../../lib/coreState/store');

  const snapshot = {
    auth: { isAuthenticated: true, userId: 'u1', user: null, profileId: null },
    sessionToken: null,
    currentUser: null,
    onboardingCompleted: welcomeLocked,
    chatOnboardingCompleted: !welcomeLocked,
    analyticsEnabled: false,
    localState: { encryptionKey: null, primaryWalletAddress: null, onboardingTasks: null },
    runtime: { screenIntelligence: null, localAi: null, autocomplete: null, service: null },
  };

  vi.mocked(useCoreState).mockReturnValue({
    snapshot,
    isBootstrapping: false,
    isReady: true,
    teams: [],
    teamMembersById: {},
    teamInvitesById: {},
    setOnboardingCompletedFlag: vi.fn(),
    setOnboardingTasks: vi.fn(),
    refreshSnapshot: vi.fn(),
  } as never);

  vi.mocked(coreStateModule.isWelcomeLocked).mockReturnValue(welcomeLocked);
  vi.mocked(coreStateModule.getCoreStateSnapshot).mockReturnValue({
    isBootstrapping: false,
    isReady: true,
    snapshot,
    teams: [],
    teamMembersById: {},
    teamInvitesById: {},
  });

  const store = buildStore({ threads, selectedThreadId, welcomeThreadId });

  // Import Conversations after mocks are wired so the module sees them
  const { default: Conversations } = await import('../Conversations');

  render(
    <Provider store={store}>
      <MemoryRouter initialEntries={['/conversations']}>
        <Conversations variant="page" />
      </MemoryRouter>
    </Provider>
  );

  return { store };
}

// ── filteredThreads ────────────────────────────────────────────────────────

describe('filteredThreads — welcome lockdown', () => {
  it('shows only the welcome thread when welcomeLocked=true and welcomeThreadId is set', async () => {
    const welcomeThread = makeThread({
      id: 'wt-1',
      title: 'Welcome',
      labels: [ONBOARDING_WELCOME_THREAD_LABEL],
    });
    const otherThread = makeThread({ id: 'other-1', title: 'Other' });

    await renderConversations({
      welcomeLocked: true,
      threads: [welcomeThread, otherThread],
      selectedThreadId: 'wt-1',
      welcomeThreadId: 'wt-1',
    });

    // The welcome thread title is replaced by "Onboarding" — see resolveThreadDisplayTitle.
    // It may appear in both the sidebar list and the header (getAllByText handles multiples).
    expect(screen.getAllByText('Onboarding').length).toBeGreaterThanOrEqual(1);
    // The other thread must not appear
    expect(screen.queryByText('Other')).not.toBeInTheDocument();
  });

  it('falls back to onboarding-labelled threads when welcomeThreadId is null but welcomeLocked=true', async () => {
    const labelledThread = makeThread({
      id: 'wt-2',
      title: 'Labelled Welcome',
      labels: [ONBOARDING_WELCOME_THREAD_LABEL],
    });
    const unlabelledThread = makeThread({ id: 'plain-1', title: 'Plain' });

    await renderConversations({
      welcomeLocked: true,
      threads: [labelledThread, unlabelledThread],
      selectedThreadId: 'wt-2',
      welcomeThreadId: null, // not yet set
    });

    // Labelled thread title is NOT replaced (welcomeThreadId is null, so the
    // label-only guard runs — it doesn't rename to "Onboarding").
    // getAllByText handles potential multi-occurrence (sidebar + header).
    expect(screen.getAllByText('Labelled Welcome').length).toBeGreaterThanOrEqual(1);
    expect(screen.queryByText('Plain')).not.toBeInTheDocument();
  });

  it('shows all threads when welcomeLocked=false', async () => {
    const thread1 = makeThread({ id: 't-1', title: 'Thread One' });
    const thread2 = makeThread({ id: 't-2', title: 'Thread Two' });

    await renderConversations({
      welcomeLocked: false,
      threads: [thread1, thread2],
      selectedThreadId: 't-1',
      welcomeThreadId: null,
    });

    expect(screen.getAllByText('Thread One').length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText('Thread Two').length).toBeGreaterThanOrEqual(1);
  });
});

// ── resolveThreadDisplayTitle ──────────────────────────────────────────────

describe('resolveThreadDisplayTitle — welcome lockdown', () => {
  it('shows "Onboarding" title for the welcome thread when locked', async () => {
    const welcomeThread = makeThread({
      id: 'wt-1',
      title: 'Do not show me',
      labels: [ONBOARDING_WELCOME_THREAD_LABEL],
    });

    await renderConversations({
      welcomeLocked: true,
      threads: [welcomeThread],
      selectedThreadId: 'wt-1',
      welcomeThreadId: 'wt-1',
    });

    // Thread list entry should read "Onboarding", not the raw server title
    expect(screen.getAllByText('Onboarding').length).toBeGreaterThanOrEqual(1);
    expect(screen.queryByText('Do not show me')).not.toBeInTheDocument();
  });

  it('shows server-side title for a non-welcome thread when NOT locked', async () => {
    const thread = makeThread({ id: 't-1', title: 'My Real Title', labels: [] });

    await renderConversations({
      welcomeLocked: false,
      threads: [thread],
      selectedThreadId: 't-1',
      welcomeThreadId: null,
    });

    expect(screen.getAllByText('My Real Title').length).toBeGreaterThanOrEqual(1);
  });

  it('shows "Onboarding" in the chat header when the welcome thread is selected and locked', async () => {
    const welcomeThread = makeThread({
      id: 'wt-1',
      title: 'Hidden Server Title',
      labels: [ONBOARDING_WELCOME_THREAD_LABEL],
    });

    await renderConversations({
      welcomeLocked: true,
      threads: [welcomeThread],
      selectedThreadId: 'wt-1',
      welcomeThreadId: 'wt-1',
    });

    // The chat header h3 also uses resolveThreadDisplayTitle
    const headerEl = document.querySelector('h3.text-sm.font-medium');
    expect(headerEl?.textContent).toBe('Onboarding');
  });

  it('returns "Select a thread" when no thread is selected', async () => {
    await renderConversations({
      welcomeLocked: false,
      threads: [],
      selectedThreadId: null,
      welcomeThreadId: null,
    });

    // Header shows the placeholder
    const headerEl = document.querySelector('h3.text-sm.font-medium');
    expect(headerEl?.textContent).toBe('Select a thread');
  });
});

// ── effectiveShowSidebar ───────────────────────────────────────────────────

describe('effectiveShowSidebar — welcome lockdown clamp', () => {
  it('sidebar is rendered (clamped open) during welcome lockdown', async () => {
    const welcomeThread = makeThread({
      id: 'wt-1',
      title: 'Welcome',
      labels: [ONBOARDING_WELCOME_THREAD_LABEL],
    });

    await renderConversations({
      welcomeLocked: true,
      threads: [welcomeThread],
      selectedThreadId: 'wt-1',
      welcomeThreadId: 'wt-1',
    });

    // Sidebar header "Threads" is rendered, proving effectiveShowSidebar=true
    expect(screen.getByText('Threads')).toBeInTheDocument();
  });

  it('sidebar can be toggled when NOT locked (showSidebar defaults to true on mount)', async () => {
    const thread = makeThread({ id: 't-1', title: 'Normal Thread' });

    await renderConversations({
      welcomeLocked: false,
      threads: [thread],
      selectedThreadId: 't-1',
      welcomeThreadId: null,
    });

    // Sidebar starts open by default
    expect(screen.getByText('Threads')).toBeInTheDocument();
  });
});

// ── Welcome thread delete button ───────────────────────────────────────────

describe('delete button visibility during welcome lockdown', () => {
  it('hides the delete button for the welcome thread when locked', async () => {
    const welcomeThread = makeThread({
      id: 'wt-1',
      title: 'Welcome',
      labels: [ONBOARDING_WELCOME_THREAD_LABEL],
    });

    await renderConversations({
      welcomeLocked: true,
      threads: [welcomeThread],
      selectedThreadId: 'wt-1',
      welcomeThreadId: 'wt-1',
    });

    expect(screen.queryByTitle('Delete thread')).not.toBeInTheDocument();
  });

  it('shows the delete button for regular threads when NOT locked', async () => {
    const thread = makeThread({ id: 't-1', title: 'Regular Thread' });

    await renderConversations({
      welcomeLocked: false,
      threads: [thread],
      selectedThreadId: 't-1',
      welcomeThreadId: null,
    });

    expect(screen.getByTitle('Delete thread')).toBeInTheDocument();
  });
});

// ── New thread / tab-bar affordances hidden during lockdown ────────────────

describe('sidebar affordances hidden during welcome lockdown', () => {
  it('hides the "New thread" button in the sidebar header when locked', async () => {
    const welcomeThread = makeThread({
      id: 'wt-1',
      title: 'Welcome',
      labels: [ONBOARDING_WELCOME_THREAD_LABEL],
    });

    await renderConversations({
      welcomeLocked: true,
      threads: [welcomeThread],
      selectedThreadId: 'wt-1',
      welcomeThreadId: 'wt-1',
    });

    expect(screen.queryByTitle('New thread')).not.toBeInTheDocument();
  });

  it('hides the label-filter tab bar during lockdown', async () => {
    const welcomeThread = makeThread({
      id: 'wt-1',
      title: 'Welcome',
      labels: [ONBOARDING_WELCOME_THREAD_LABEL],
    });

    await renderConversations({
      welcomeLocked: true,
      threads: [welcomeThread],
      selectedThreadId: 'wt-1',
      welcomeThreadId: 'wt-1',
    });

    expect(screen.queryByTestId('pill-tab-bar')).not.toBeInTheDocument();
  });

  it('shows "New thread" button and tab bar when NOT locked', async () => {
    const thread = makeThread({ id: 't-1', title: 'Regular' });

    await renderConversations({
      welcomeLocked: false,
      threads: [thread],
      selectedThreadId: 't-1',
      welcomeThreadId: null,
    });

    expect(screen.getByTitle('New thread')).toBeInTheDocument();
    expect(screen.getByTestId('pill-tab-bar')).toBeInTheDocument();
  });
});
