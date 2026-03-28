// Global type declarations for the application

declare global {
  interface Window {
    __TAURI__?: { [key: string]: unknown };
  }
}

export {};
