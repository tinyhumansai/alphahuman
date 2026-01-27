import { TelegramClient } from 'telegram';
import { StringSession } from 'telegram/sessions';
import type { UserAuthParams, BotAuthParams } from 'telegram/client/auth';
import { TELEGRAM_API_ID, TELEGRAM_API_HASH } from '../utils/config';

type LoginOptions = UserAuthParams | BotAuthParams;

class MTProtoService {
  private static instance: MTProtoService | undefined;
  private client: TelegramClient | undefined;
  private isInitialized = false;
  private isConnected = false;
  private sessionString = '';
  private readonly apiId: number;
  private readonly apiHash: string;

  private constructor() {
    // Private constructor to enforce singleton
    // Load API credentials from config once
    if (!TELEGRAM_API_ID || !TELEGRAM_API_HASH) {
      throw new Error('TELEGRAM_API_ID and TELEGRAM_API_HASH must be configured');
    }
    this.apiId = TELEGRAM_API_ID;
    this.apiHash = TELEGRAM_API_HASH;
  }

  static getInstance(): MTProtoService {
    if (!MTProtoService.instance) {
      MTProtoService.instance = new MTProtoService();
    }
    return MTProtoService.instance;
  }

  /**
   * Initialize the MTProto client with API credentials
   */
  async initialize(): Promise<void> {
    if (this.isInitialized && this.client) {
      console.log('MTProto client already initialized');
      return;
    }

    const sessionString = this.loadSession() || '';

    try {
      const stringSession = new StringSession(sessionString);
      this.sessionString = sessionString;

      this.client = new TelegramClient(stringSession, this.apiId, this.apiHash, {
        connectionRetries: 5,
      });

      this.isInitialized = true;
      console.log('MTProto client initialized successfully');
    } catch (error) {
      console.error('Failed to initialize MTProto client:', error);
      throw error;
    }
  }

  /**
   * Connect to Telegram servers
   */
  async connect(): Promise<void> {
    if (!this.client) {
      throw new Error('MTProto client not initialized. Call initialize() first.');
    }

    if (this.isConnected) {
      console.log('Already connected to Telegram');
      return;
    }

    try {
      await this.client.connect();
      this.isConnected = true;
      console.log('Connected to Telegram successfully');

      // Save session string if it changed
      const newSessionString = this.client.session.save();
      if (newSessionString && newSessionString !== this.sessionString) {
        this.sessionString = newSessionString;
        this.saveSession(newSessionString);
        console.log('Session updated and saved');
      }
    } catch (error) {
      console.error('Failed to connect to Telegram:', error);
      throw error;
    }
  }

  /**
   * Start authentication/login process
   */
  async start(options: LoginOptions): Promise<void> {
    if (!this.client) {
      throw new Error('MTProto client not initialized. Call initialize() first.');
    }

    try {
      await this.client.start(options);

      // Save session after successful login
      const newSessionString = this.client.session.save();
      if (newSessionString && newSessionString !== this.sessionString) {
        this.sessionString = newSessionString;
        this.saveSession(newSessionString);
        console.log('Authentication successful, session saved');
      }
    } catch (error) {
      console.error('Authentication failed:', error);
      throw error;
    }
  }

  /**
   * Sign in using QR code
   */
  async signInWithQrCode(
    qrCodeCallback: (qrCode: { token: Buffer; expires: number }) => void,
    passwordCallback?: (hint?: string) => Promise<string>,
    onError?: (err: Error) => Promise<boolean> | void
  ): Promise<unknown> {
    if (!this.client) {
      throw new Error('MTProto client not initialized. Call initialize() first.');
    }

    try {
      const user = await this.client.signInUserWithQrCode(
        {
          apiId: this.apiId,
          apiHash: this.apiHash,
        },
        {
          qrCode: async (qrCode) => {
            qrCodeCallback(qrCode);
          },
          password: passwordCallback,
          onError: onError || ((err: Error) => {
            console.error('QR code auth error:', err);
            return false;
          }),
        }
      );

      // Save session after successful login
      const newSessionString = this.client.session.save();
      if (newSessionString && newSessionString !== this.sessionString) {
        this.sessionString = newSessionString;
        this.saveSession(newSessionString);
        console.log('QR code authentication successful, session saved');
      }

      return user;
    } catch (error) {
      console.error('QR code authentication failed:', error);
      throw error;
    }
  }

  /**
   * Get the Telegram client instance
   * @throws Error if client is not initialized
   */
  getClient(): TelegramClient {
    if (!this.client || !this.isInitialized) {
      throw new Error('MTProto client not initialized. Call initialize() first.');
    }
    return this.client;
  }

  /**
   * Check if the client is initialized
   */
  isReady(): boolean {
    return this.isInitialized && this.client !== undefined;
  }

  /**
   * Check if the client is connected
   */
  isClientConnected(): boolean {
    return this.isConnected && this.isReady();
  }

  /**
   * Get the current session string
   */
  getSessionString(): string {
    return this.sessionString;
  }

  /**
   * Disconnect from Telegram
   */
  async disconnect(): Promise<void> {
    if (this.client && this.isConnected) {
      try {
        await this.client.disconnect();
        this.isConnected = false;
        console.log('Disconnected from Telegram');
      } catch (error) {
        console.error('Error disconnecting from Telegram:', error);
        throw error;
      }
    }
  }

  /**
   * Send a message using the client
   */
  async sendMessage(entity: string, message: string): Promise<void> {
    const client = this.getClient();
    if (!this.isClientConnected()) {
      await this.connect();
    }
    await client.sendMessage(entity, { message });
  }

  /**
   * Invoke a raw Telegram API method
   */
  async invoke<T = unknown>(request: Parameters<TelegramClient['invoke']>[0]): Promise<T> {
    const client = this.getClient();
    if (!this.isClientConnected()) {
      await this.connect();
    }
    return client.invoke(request) as Promise<T>;
  }

  /**
   * Load session from localStorage
   */
  private loadSession(): string | null {
    try {
      return localStorage.getItem('telegram_session');
    } catch (error) {
      console.error('Failed to load session from localStorage:', error);
      return null;
    }
  }

  /**
   * Save session to localStorage
   */
  private saveSession(session: string): void {
    try {
      localStorage.setItem('telegram_session', session);
    } catch (error) {
      console.error('Failed to save session to localStorage:', error);
    }
  }
}

// Export singleton instance
export const mtprotoService = MTProtoService.getInstance();
export default mtprotoService;
