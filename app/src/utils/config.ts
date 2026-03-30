export const CORE_RPC_URL =
  import.meta.env.OPENHUMAN_CORE_RPC_URL ||
  import.meta.env.VITE_OPENHUMAN_CORE_RPC_URL ||
  'http://127.0.0.1:7788/rpc';

export const IS_DEV = import.meta.env.DEV;

export const SKILLS_GITHUB_REPO =
  import.meta.env.VITE_SKILLS_GITHUB_REPO || 'tinyhumansai/openhuman-skills';
