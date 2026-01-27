export const BACKEND_URL =
  import.meta.env.VITE_BACKEND_URL || "https://api.alphahuman.xyz";

export const TELEGRAM_BOT_USERNAME =
  import.meta.env.VITE_TELEGRAM_BOT_USERNAME || "alphahumanx_bot";

export const TELEGRAM_BOT_ID =
  import.meta.env.VITE_TELEGRAM_BOT_ID || "8043922470";

export const IS_DEV =
  Boolean(import.meta.env.DEV) || import.meta.env.MODE === "development";
