/**
 * Tests for app/src-tauri/recipes/google-meet/agent.js pure helpers.
 *
 * We load the agent script in a fresh jsdom window via Function() evaluation,
 * set up the role="agent" context, and extract `window.__openhumanMeetAgent.pure`
 * for unit testing. The polling loop is never started in this environment because
 * we don't provide a real meetingUrl navigation context.
 */
import fs from 'fs';
import path from 'path';
import { beforeEach, describe, expect, it } from 'vitest';

// Read the agent.js source once.
const AGENT_JS_PATH = path.resolve(__dirname, '../src-tauri/recipes/google-meet/agent.js');
const agentSource = fs.readFileSync(AGENT_JS_PATH, 'utf8');

type PureHelpers = {
  extractMeetingCode: (href: string) => string | null;
  queryByCssOrText: (doc: Document, selector: string) => Element | null;
  queryAllByCssOrText: (doc: Document, selector: string) => Element[];
  firstFromList: (doc: Document, selectors: string[]) => Element | null;
  findJoinButton: (doc: Document) => Element | null;
  findMicButton: (doc: Document) => Element | null;
  findCamButton: (doc: Document) => Element | null;
  isMicOn: (btn: Element | null) => boolean;
  isCamOn: (btn: Element | null) => boolean;
  isInCall: (doc: Document) => boolean;
  isInWaitingRoom: (doc: Document) => boolean;
  findLeaveButton: (doc: Document) => Element | null;
  isUnjoinableScreen: (doc: Document) => string | null;
};

/**
 * Evaluate agent.js in the current jsdom window with a mock context and API.
 * Returns the `pure` namespace from `window.__openhumanMeetAgent`.
 */
function loadAgent(meetingUrl = 'https://meet.google.com/abc-defg-hij') {
  // Reset any prior agent state.
  delete (window as Window & { __openhumanMeetAgent?: unknown }).__openhumanMeetAgent;

  // Set up the recipe context with role="agent".
  (window as Window & { __OPENHUMAN_RECIPE_CTX__?: unknown }).__OPENHUMAN_RECIPE_CTX__ = {
    accountId: 'test-account',
    provider: 'google-meet',
    role: 'agent',
    meetingUrl,
  };

  // Minimal mock of the runtime API (emit + log).
  const emitted: Array<{ kind: string; payload: unknown }> = [];
  (window as Window & { __openhumanRecipe?: unknown }).__openhumanRecipe = {
    emit: (kind: string, payload: unknown) => emitted.push({ kind, payload }),
    log: () => {},
    loop: () => {},
  };

  // Run the agent script.
  // Function constructor used intentionally to evaluate the agent script in the jsdom context.
  // biome-ignore lint: intentional use of Function constructor for test harness
  new Function(agentSource)();

  const agent = (window as Window & { __openhumanMeetAgent?: { pure: Record<string, unknown> } })
    .__openhumanMeetAgent;

  if (!agent) throw new Error('__openhumanMeetAgent not set after loading agent.js');

  return agent.pure as PureHelpers;
}

// ─── extractMeetingCode ────────────────────────────────────────────────────

describe('extractMeetingCode', () => {
  let pure: PureHelpers;

  beforeEach(() => {
    pure = loadAgent();
  });

  it('extracts a standard 3-part code', () => {
    expect(pure.extractMeetingCode('https://meet.google.com/abc-defg-hij')).toBe('abc-defg-hij');
  });

  it('extracts code with trailing slash', () => {
    expect(pure.extractMeetingCode('https://meet.google.com/abc-defg-hij/')).toBe('abc-defg-hij');
  });

  it('extracts code with query string', () => {
    expect(pure.extractMeetingCode('https://meet.google.com/abc-defg-hij?authuser=0')).toBe(
      'abc-defg-hij'
    );
  });

  it('returns null for a non-meeting URL', () => {
    expect(pure.extractMeetingCode('https://meet.google.com/')).toBeNull();
  });

  it('returns null for empty string', () => {
    expect(pure.extractMeetingCode('')).toBeNull();
  });

  it('returns null for a URL with no matching pathname', () => {
    expect(pure.extractMeetingCode('https://meet.google.com/settings')).toBeNull();
  });
});

// ─── queryByCssOrText / queryAllByCssOrText / firstFromList ───────────────

describe('queryByCssOrText — plain CSS', () => {
  let pure: PureHelpers;

  beforeEach(() => {
    pure = loadAgent();
    document.body.innerHTML = '';
  });

  it('finds an element by plain CSS selector', () => {
    document.body.innerHTML = '<button aria-label="Leave call">Leave</button>';
    const el = pure.queryByCssOrText(document, 'button[aria-label="Leave call"]');
    expect(el).not.toBeNull();
  });

  it('returns null when CSS selector has no match', () => {
    document.body.innerHTML = '<button>Settings</button>';
    expect(pure.queryByCssOrText(document, 'button[aria-label="Leave call"]')).toBeNull();
  });

  it('queryAllByCssOrText returns all CSS matches', () => {
    document.body.innerHTML =
      '<button aria-label="Leave call">x</button><button aria-label="Leave call">y</button>';
    expect(pure.queryAllByCssOrText(document, 'button[aria-label="Leave call"]').length).toBe(2);
  });
});

describe('queryByCssOrText — :has-text form', () => {
  let pure: PureHelpers;

  beforeEach(() => {
    pure = loadAgent();
    document.body.innerHTML = '';
  });

  it('finds button by :has-text("Join now")', () => {
    document.body.innerHTML = '<button>Join now</button>';
    const el = pure.queryByCssOrText(document, 'button:has-text("Join now")');
    expect(el).not.toBeNull();
    expect((el as HTMLElement).tagName.toLowerCase()).toBe('button');
  });

  it('is case-insensitive for :has-text text match', () => {
    document.body.innerHTML = '<button>JOIN NOW</button>';
    const el = pure.queryByCssOrText(document, 'button:has-text("Join now")');
    expect(el).not.toBeNull();
  });

  it('returns null when :has-text text is not present', () => {
    document.body.innerHTML = '<button>Settings</button>';
    expect(pure.queryByCssOrText(document, 'button:has-text("Join now")')).toBeNull();
  });

  it(':has-text with no CSS prefix matches any element', () => {
    document.body.innerHTML = '<span>Ask to join</span>';
    const el = pure.queryByCssOrText(document, ':has-text("Ask to join")');
    expect(el).not.toBeNull();
  });
});

describe('queryByCssOrText — XPath form', () => {
  let pure: PureHelpers;

  beforeEach(() => {
    pure = loadAgent();
    document.body.innerHTML = '';
  });

  it('finds button via XPath with descendant span text', () => {
    document.body.innerHTML = '<button><span>Ask to join</span></button>';
    const el = pure.queryByCssOrText(document, '//button[.//span[text()="Ask to join"]]');
    expect(el).not.toBeNull();
    expect((el as HTMLElement).tagName.toLowerCase()).toBe('button');
  });

  it('returns null when XPath has no match', () => {
    document.body.innerHTML = '<button><span>Join now</span></button>';
    const el = pure.queryByCssOrText(document, '//button[.//span[text()="Ask to join"]]');
    expect(el).toBeNull();
  });
});

describe('queryByCssOrText — text= (exact) form', () => {
  let pure: PureHelpers;

  beforeEach(() => {
    pure = loadAgent();
    document.body.innerHTML = '';
  });

  it('finds element with exact text match', () => {
    document.body.innerHTML = '<div>Asking to be let in...</div>';
    const el = pure.queryByCssOrText(document, 'text="Asking to be let in..."');
    expect(el).not.toBeNull();
  });

  it('does not match partial text with text= form', () => {
    document.body.innerHTML = '<div>Asking to be let in... please wait</div>';
    // "Asking to be let in..." is a substring, not the exact textContent of the div
    // (the full text is longer); text= requires the full trimmed text to match.
    const el = pure.queryByCssOrText(document, 'text="Asking to be let in..."');
    expect(el).toBeNull();
  });
});

describe('queryByCssOrText — text*= (contains) form', () => {
  let pure: PureHelpers;

  beforeEach(() => {
    pure = loadAgent();
    document.body.innerHTML = '';
  });

  it('finds element when text contains substring', () => {
    document.body.innerHTML = "<div>You'll join the call when someone lets you in soon</div>";
    const el = pure.queryByCssOrText(
      document,
      'text*="You\'ll join the call when someone lets you"'
    );
    expect(el).not.toBeNull();
  });

  it('returns null when substring is not present', () => {
    document.body.innerHTML = '<div>Joining...</div>';
    expect(
      pure.queryByCssOrText(document, 'text*="You\'ll join the call when someone lets you"')
    ).toBeNull();
  });
});

describe('firstFromList', () => {
  let pure: PureHelpers;

  beforeEach(() => {
    pure = loadAgent();
    document.body.innerHTML = '';
  });

  it('returns first matching selector', () => {
    document.body.innerHTML = '<button aria-label="Leave call">Leave</button>';
    const el = pure.firstFromList(document, [
      'button[aria-label="foo"]',
      'button[aria-label="Leave call"]',
      'button[aria-label="bar"]',
    ]);
    expect(el).not.toBeNull();
  });

  it('falls through unmatched selectors to the matching one', () => {
    // Only the third selector matches — verifies fallthrough.
    document.body.innerHTML = '<button aria-label="Turn on camera">cam</button>';
    const el = pure.firstFromList(document, [
      'button[aria-label="Leave call"]',
      'button[aria-label="Turn off camera"]',
      'button[aria-label="Turn on camera"]',
    ]);
    expect(el).not.toBeNull();
    expect((el as HTMLElement).getAttribute('aria-label')).toBe('Turn on camera');
  });

  it('returns null when no selector in the list matches', () => {
    document.body.innerHTML = '<div>Nothing here</div>';
    expect(
      pure.firstFromList(document, ['button[aria-label="Leave call"]', 'button:has-text("Join")'])
    ).toBeNull();
  });
});

// ─── findJoinButton ────────────────────────────────────────────────────────

describe('findJoinButton', () => {
  let pure: PureHelpers;

  beforeEach(() => {
    pure = loadAgent();
    document.body.innerHTML = '';
  });

  it('finds button via XPath (first selector in list)', () => {
    document.body.innerHTML = '<button><span>Ask to join</span></button>';
    expect(pure.findJoinButton(document)).not.toBeNull();
  });

  it('finds button via :has-text("Ask to join") (second selector)', () => {
    // No span child — XPath won't match, :has-text should.
    document.body.innerHTML = '<button>Ask to join</button>';
    expect(pure.findJoinButton(document)).not.toBeNull();
  });

  it('finds button via :has-text("Join now") (third selector) when earlier selectors miss', () => {
    document.body.innerHTML = '<button>Join now</button>';
    expect(pure.findJoinButton(document)).not.toBeNull();
  });

  it('finds button via :has-text("Join") (fourth / last selector)', () => {
    document.body.innerHTML = '<button>Join</button>';
    expect(pure.findJoinButton(document)).not.toBeNull();
  });

  it('returns null when only a disabled button is present', () => {
    document.body.innerHTML = '<button disabled>Join now</button>';
    expect(pure.findJoinButton(document)).toBeNull();
  });

  it('returns null when no matching button exists', () => {
    document.body.innerHTML = '<button>Settings</button>';
    expect(pure.findJoinButton(document)).toBeNull();
  });

  it('skips disabled buttons when a non-disabled match exists later', () => {
    document.body.innerHTML = '<button disabled>Join now</button><button>Join now</button>';
    const btn = pure.findJoinButton(document);
    expect(btn).not.toBeNull();
    expect((btn as HTMLButtonElement).disabled).toBe(false);
  });
});

// ─── findMicButton / isMicOn ───────────────────────────────────────────────

describe('findMicButton + isMicOn', () => {
  let pure: PureHelpers;

  beforeEach(() => {
    pure = loadAgent();
    document.body.innerHTML = '';
  });

  it('finds mic button by "Turn off microphone" aria-label', () => {
    document.body.innerHTML = '<button aria-label="Turn off microphone">mic</button>';
    expect(pure.findMicButton(document)).not.toBeNull();
  });

  it('finds mic button by "Turn on microphone" aria-label', () => {
    document.body.innerHTML = '<button aria-label="Turn on microphone">mic</button>';
    expect(pure.findMicButton(document)).not.toBeNull();
  });

  it('isMicOn returns true when aria-label says "Turn off microphone"', () => {
    document.body.innerHTML = '<button aria-label="Turn off microphone">mic</button>';
    const btn = pure.findMicButton(document);
    expect(pure.isMicOn(btn)).toBe(true);
  });

  it('isMicOn returns false when aria-label says "Turn on microphone"', () => {
    document.body.innerHTML = '<button aria-label="Turn on microphone">mic</button>';
    const btn = pure.findMicButton(document);
    expect(pure.isMicOn(btn)).toBe(false);
  });

  it('isMicOn falls back to aria-pressed="true"', () => {
    document.body.innerHTML = '<button aria-label="microphone" aria-pressed="true">mic</button>';
    const btn = document.querySelector('button') as Element;
    expect(pure.isMicOn(btn)).toBe(true);
  });

  it('isMicOn falls back to data-is-muted="false"', () => {
    document.body.innerHTML = '<button aria-label="microphone" data-is-muted="false">mic</button>';
    const btn = document.querySelector('button') as Element;
    expect(pure.isMicOn(btn)).toBe(true);
  });

  it('isMicOn defaults to false on ambiguous node', () => {
    document.body.innerHTML = '<button aria-label="microphone">mic</button>';
    const btn = pure.findMicButton(document);
    expect(pure.isMicOn(btn)).toBe(false);
  });

  it('isMicOn returns false for null', () => {
    expect(pure.isMicOn(null)).toBe(false);
  });
});

// ─── findCamButton / isCamOn ───────────────────────────────────────────────

describe('findCamButton + isCamOn', () => {
  let pure: PureHelpers;

  beforeEach(() => {
    pure = loadAgent();
    document.body.innerHTML = '';
  });

  it('finds cam button by "Turn off camera" aria-label', () => {
    document.body.innerHTML = '<button aria-label="Turn off camera">cam</button>';
    expect(pure.findCamButton(document)).not.toBeNull();
  });

  it('isCamOn returns true when aria-label says "Turn off camera"', () => {
    document.body.innerHTML = '<button aria-label="Turn off camera">cam</button>';
    const btn = pure.findCamButton(document);
    expect(pure.isCamOn(btn)).toBe(true);
  });

  it('isCamOn returns false when aria-label says "Turn on camera"', () => {
    document.body.innerHTML = '<button aria-label="Turn on camera">cam</button>';
    const btn = pure.findCamButton(document);
    expect(pure.isCamOn(btn)).toBe(false);
  });

  it('isCamOn returns false for null', () => {
    expect(pure.isCamOn(null)).toBe(false);
  });
});

// ─── isInCall ──────────────────────────────────────────────────────────────

describe('isInCall', () => {
  let pure: PureHelpers;

  beforeEach(() => {
    pure = loadAgent();
    document.body.innerHTML = '';
  });

  it('returns true when data-self-name is present (initialAdmissionIndicators)', () => {
    document.body.innerHTML = '<div data-self-name="Alice"></div>';
    expect(pure.isInCall(document)).toBe(true);
  });

  it('returns true when data-participant-id is present (initialAdmissionIndicators)', () => {
    document.body.innerHTML = '<div data-participant-id="part-123"></div>';
    expect(pure.isInCall(document)).toBe(true);
  });

  it('returns true for Share screen button (initialAdmissionIndicators)', () => {
    document.body.innerHTML = '<button aria-label="Share screen">share</button>';
    expect(pure.isInCall(document)).toBe(true);
  });

  it('returns true for Present now button (initialAdmissionIndicators)', () => {
    document.body.innerHTML = '<button aria-label="Present now">present</button>';
    expect(pure.isInCall(document)).toBe(true);
  });

  it('returns true for "Leave call" button (admissionIndicators)', () => {
    document.body.innerHTML = '<button aria-label="Leave call">leave</button>';
    expect(pure.isInCall(document)).toBe(true);
  });

  it('returns true for toolbar (admissionIndicators)', () => {
    document.body.innerHTML = '<div role="toolbar"></div>';
    expect(pure.isInCall(document)).toBe(true);
  });

  it('returns false when neither signal is present', () => {
    document.body.innerHTML = '<div>Lobby</div>';
    expect(pure.isInCall(document)).toBe(false);
  });
});

// ─── isInWaitingRoom ──────────────────────────────────────────────────────

describe('isInWaitingRoom', () => {
  let pure: PureHelpers;

  beforeEach(() => {
    pure = loadAgent();
    document.body.innerHTML = '';
  });

  it('returns true for "Asking to be let in..." text', () => {
    document.body.innerHTML = '<div>Asking to be let in...</div>';
    expect(pure.isInWaitingRoom(document)).toBe(true);
  });

  it('returns true for "You\'ll join the call when someone lets you in" text', () => {
    document.body.innerHTML = "<div>You'll join the call when someone lets you in</div>";
    expect(pure.isInWaitingRoom(document)).toBe(true);
  });

  it('returns true for partial match via text*= selector ("Asking to be let in")', () => {
    document.body.innerHTML = '<div>Asking to be let in — please hold</div>';
    expect(pure.isInWaitingRoom(document)).toBe(true);
  });

  it('returns true for aria-label containing "waiting room"', () => {
    document.body.innerHTML = '<div aria-label="waiting room status"></div>';
    expect(pure.isInWaitingRoom(document)).toBe(true);
  });

  it('returns false when not in waiting room', () => {
    document.body.innerHTML = '<div>Join now</div>';
    expect(pure.isInWaitingRoom(document)).toBe(false);
  });
});

// ─── isUnjoinableScreen ────────────────────────────────────────────────────

describe('isUnjoinableScreen', () => {
  let pure: PureHelpers;

  beforeEach(() => {
    pure = loadAgent();
    document.body.innerHTML = '';
  });

  it('returns "meeting-not-found" for "Meeting not found" text', () => {
    document.body.innerHTML = '<p>Meeting not found</p>';
    expect(pure.isUnjoinableScreen(document)).toBe('meeting-not-found');
  });

  it('returns "meeting-ended" for "Meeting has ended" text', () => {
    document.body.innerHTML = '<p>Meeting has ended</p>';
    expect(pure.isUnjoinableScreen(document)).toBe('meeting-ended');
  });

  it('returns "meeting-ended" for "This meeting has ended" text', () => {
    document.body.innerHTML = '<p>This meeting has ended</p>';
    expect(pure.isUnjoinableScreen(document)).toBe('meeting-ended');
  });

  it('returns "access-denied" for "Can\'t join the meeting" text', () => {
    document.body.innerHTML = "<p>Can't join the meeting</p>";
    expect(pure.isUnjoinableScreen(document)).toBe('access-denied');
  });

  it('returns "access-denied" for "Access denied" text', () => {
    document.body.innerHTML = '<p>Access denied</p>';
    expect(pure.isUnjoinableScreen(document)).toBe('access-denied');
  });

  it('returns "invalid-link" for "Invalid meeting" text', () => {
    document.body.innerHTML = '<p>Invalid meeting</p>';
    expect(pure.isUnjoinableScreen(document)).toBe('invalid-link');
  });

  it('returns "invalid-link" for "Meeting link expired" text', () => {
    document.body.innerHTML = '<p>Meeting link expired</p>';
    expect(pure.isUnjoinableScreen(document)).toBe('invalid-link');
  });

  it('returns "unable-to-join" for "Unable to join" text', () => {
    document.body.innerHTML = '<p>Unable to join</p>';
    expect(pure.isUnjoinableScreen(document)).toBe('unable-to-join');
  });

  it('returns "unable-to-join" for "Try again" button', () => {
    document.body.innerHTML = '<button>Try again</button>';
    expect(pure.isUnjoinableScreen(document)).toBe('unable-to-join');
  });

  it('returns null when none of the patterns match', () => {
    document.body.innerHTML = '<div>Joining meeting...</div>';
    expect(pure.isUnjoinableScreen(document)).toBeNull();
  });
});

// ─── findLeaveButton ───────────────────────────────────────────────────────

describe('findLeaveButton', () => {
  let pure: PureHelpers;

  beforeEach(() => {
    pure = loadAgent();
    document.body.innerHTML = '';
  });

  it('finds primary leave button by exact aria-label "Leave call"', () => {
    document.body.innerHTML = '<button aria-label="Leave call">Leave</button>';
    expect(pure.findLeaveButton(document)).not.toBeNull();
  });

  it('finds primary leave button by partial aria-label "Leave"', () => {
    document.body.innerHTML = '<button aria-label="Leave meeting">Leave</button>';
    expect(pure.findLeaveButton(document)).not.toBeNull();
  });

  it('finds primary leave via toolbar button with Leave aria-label', () => {
    document.body.innerHTML =
      '<div role="toolbar"><button aria-label="Leave call">Leave</button></div>';
    expect(pure.findLeaveButton(document)).not.toBeNull();
  });

  it('finds secondary leave button via :has-text("Leave meeting") when primary is absent', () => {
    // No primary leave aria-label — only secondary confirmation dialog text.
    document.body.innerHTML = '<div role="dialog"><button>Leave meeting</button></div>';
    expect(pure.findLeaveButton(document)).not.toBeNull();
  });

  it('finds secondary leave via :has-text("Just leave the meeting")', () => {
    document.body.innerHTML = '<button>Just leave the meeting</button>';
    expect(pure.findLeaveButton(document)).not.toBeNull();
  });

  it('returns null when no leave button present', () => {
    document.body.innerHTML = '<button>Join now</button>';
    expect(pure.findLeaveButton(document)).toBeNull();
  });
});

// ─── Role gate ─────────────────────────────────────────────────────────────

describe('role gate', () => {
  it('does not set __openhumanMeetAgent when role is not "agent"', () => {
    // @ts-expect-error - test harness
    delete (window as Window).__openhumanMeetAgent;

    (window as Window & { __OPENHUMAN_RECIPE_CTX__?: unknown }).__OPENHUMAN_RECIPE_CTX__ = {
      accountId: 'test-account',
      provider: 'google-meet',
      role: 'user', // NOT agent
      meetingUrl: 'https://meet.google.com/abc-defg-hij',
    };

    // Function constructor used intentionally for test harness evaluation.
    // biome-ignore lint: intentional use of Function constructor for test harness
    new Function(agentSource)();

    expect(
      (window as Window & { __openhumanMeetAgent?: unknown }).__openhumanMeetAgent
    ).toBeUndefined();
  });
});
