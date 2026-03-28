export const BACKEND_URL = import.meta.env.VITE_BACKEND_URL || 'https://api.tinyhumans.ai';

export const IS_DEV = import.meta.env.DEV;

export const SKILLS_GITHUB_REPO =
  import.meta.env.VITE_SKILLS_GITHUB_REPO || 'tinyhumansai/openhuman-skills';

export const DEV_AUTO_LOAD_SKILL = import.meta.env.VITE_DEV_AUTO_LOAD_SKILL || undefined;
