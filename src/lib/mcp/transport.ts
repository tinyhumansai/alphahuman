/**
 * Socket.IO transport for MCP
 * Handles communication between frontend MCP server and backend MCP client
 */

import type { Socket } from 'socket.io-client';
import type { MCPRequest, MCPResponse, SocketIOMCPTransport } from './types';
import { mcpWarn } from './logger';

export class SocketIOMCPTransportImpl implements SocketIOMCPTransport {
  private socket: Socket | null | undefined;
  private requestHandlers = new Map<string | number, (response: MCPResponse) => void>();
  private readonly eventPrefix = 'mcp:';
  private responseHandler = (response: MCPResponse): void => {
    const handler = this.requestHandlers.get(response.id);
    if (handler) {
      handler(response);
      this.requestHandlers.delete(response.id);
    }
  };

  constructor(socket: Socket | null | undefined) {
    this.socket = socket ?? undefined;
    this.setupEventHandlers();
  }

  get connected(): boolean {
    return Boolean(this.socket?.connected);
  }

  private setupEventHandlers(): void {
    if (!this.socket) return;
    this.socket.on(`${this.eventPrefix}response`, this.responseHandler);
  }

  emit(event: string, data: unknown): void {
    if (!this.socket?.connected) {
      mcpWarn('Cannot emit MCP event: socket not connected', { event });
      return;
    }
    this.socket.emit(`${this.eventPrefix}${event}`, data);
  }

  on(event: string, handler: (data: unknown) => void): void {
    if (!this.socket) return;
    this.socket.on(`${this.eventPrefix}${event}`, handler);
  }

  off(event: string, handler: (data: unknown) => void): void {
    if (!this.socket) return;
    this.socket.off(`${this.eventPrefix}${event}`, handler);
  }

  async request(request: MCPRequest, timeoutMs = 30000): Promise<MCPResponse> {
    if (!this.socket?.connected) {
      throw new Error('Socket not connected');
    }

    return new Promise<MCPResponse>((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.requestHandlers.delete(request.id);
        reject(new Error(`MCP request timeout after ${timeoutMs}ms`));
      }, timeoutMs);

      this.requestHandlers.set(request.id, (response: MCPResponse) => {
        clearTimeout(timeout);
        if (response.error) {
          reject(new Error(response.error.message));
        } else {
          resolve(response);
        }
      });

      this.emit('request', request);
    });
  }

  updateSocket(socket: Socket | null | undefined): void {
    if (this.socket) {
      this.socket.off(`${this.eventPrefix}response`, this.responseHandler);
    }
    this.socket = socket ?? undefined;
    this.setupEventHandlers();
  }
}
