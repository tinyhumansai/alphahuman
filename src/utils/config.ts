export const BACKEND_URL = import.meta.env.VITE_BACKEND_URL || 'https://api.tinyhumans.ai';

export const IS_DEV = import.meta.env.DEV;

// Enforced mode: always use HTTP chat only (no backend socket orchestration).
export const HTTP_CHAT_ONLY = true;

export const SKILLS_GITHUB_REPO =
  import.meta.env.VITE_SKILLS_GITHUB_REPO || 'tinyhumansai/openhuman-skills';

export const DEV_AUTO_LOAD_SKILL = import.meta.env.VITE_DEV_AUTO_LOAD_SKILL || undefined;
