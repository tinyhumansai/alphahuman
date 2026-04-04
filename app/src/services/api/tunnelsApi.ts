import type { ApiResponse } from '../../types/api';
import { apiClient } from '../apiClient';

// ── Types ─────────────────────────────────────────────────────────────────────

export interface Tunnel {
  /** Internal backend ID (used for CRUD endpoints: GET/PATCH/DELETE /webhooks/core/:id). */
  id: string;
  /** External UUID used for ingress routing (appears in webhook URLs and local registrations). */
  uuid: string;
  name: string;
  description?: string;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface TunnelBandwidthUsage {
  remainingBudgetUsd: number;
}

export interface CreateTunnelRequest {
  name: string;
  description?: string;
}

export interface UpdateTunnelRequest {
  name?: string;
  description?: string;
  isActive?: boolean;
}

// ── API ───────────────────────────────────────────────────────────────────────

export const tunnelsApi = {
  /** POST /webhooks/core — create a new webhook tunnel */
  createTunnel: async (body: CreateTunnelRequest): Promise<Tunnel> => {
    const response = await apiClient.post<ApiResponse<Tunnel>>('/webhooks/core', body);
    return response.data;
  },

  /** GET /webhooks/core — list user's webhook tunnels */
  getTunnels: async (): Promise<Tunnel[]> => {
    const response = await apiClient.get<ApiResponse<Tunnel[]>>('/webhooks/core');
    return response.data;
  },

  /** GET /webhooks/core/bandwidth — get remaining webhook bandwidth budget */
  getBandwidthUsage: async (): Promise<TunnelBandwidthUsage> => {
    const response = await apiClient.get<ApiResponse<TunnelBandwidthUsage>>(
      '/webhooks/core/bandwidth'
    );
    return response.data;
  },

  /** GET /webhooks/core/:tunnelId — get a specific webhook tunnel by its internal ID. */
  getTunnel: async (tunnelId: string): Promise<Tunnel> => {
    const response = await apiClient.get<ApiResponse<Tunnel>>(`/webhooks/core/${tunnelId}`);
    return response.data;
  },

  /** PATCH /webhooks/core/:tunnelId — update a webhook tunnel by its internal ID. */
  updateTunnel: async (tunnelId: string, body: UpdateTunnelRequest): Promise<Tunnel> => {
    const response = await apiClient.patch<ApiResponse<Tunnel>>(
      `/webhooks/core/${tunnelId}`,
      body
    );
    return response.data;
  },

  /** DELETE /webhooks/core/:tunnelId — delete a webhook tunnel by its internal ID. */
  deleteTunnel: async (tunnelId: string): Promise<void> => {
    await apiClient.delete<ApiResponse<unknown>>(`/webhooks/core/${tunnelId}`);
  },
};
