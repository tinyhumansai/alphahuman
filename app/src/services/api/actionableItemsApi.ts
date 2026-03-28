import type { ApiResponse } from '../../types/api';
import { apiClient } from '../apiClient';

interface ActionableItem {
  id: string;
  title: string;
  description?: string;
  status: 'pending' | 'dismissed' | 'snoozed' | 'completed';
  createdAt: string;
  updatedAt: string;
  // Add other fields based on backend schema
}

interface ExecutionSession {
  id: string;
  itemId: string;
  status: 'running' | 'pending' | 'completed' | 'failed';
  // Add other fields based on backend schema
}

interface ThreadData {
  threadId: string;
  conversationId: string;
}

/**
 * Actionable Items API endpoints
 */
export const actionableItemsApi = {
  /**
   * List actionable items for the authenticated user
   * GET /telegram/actionable-items
   */
  getActionableItems: async (): Promise<ActionableItem[]> => {
    const response = await apiClient.get<ApiResponse<ActionableItem[]>>(
      '/telegram/actionable-items'
    );
    return response.data;
  },

  /**
   * Update an actionable item (dismiss, snooze, etc.)
   * PATCH /telegram/actionable-items/:itemId
   */
  updateActionableItem: async (
    itemId: string,
    updates: { status?: ActionableItem['status']; snoozeUntil?: string }
  ): Promise<ActionableItem> => {
    const response = await apiClient.patch<ApiResponse<ActionableItem>>(
      `/telegram/actionable-items/${itemId}`,
      updates
    );
    return response.data;
  },

  /**
   * Get or create conversation thread for an actionable item
   * GET /telegram/actionable-items/:itemId/thread
   */
  getItemThread: async (itemId: string): Promise<ThreadData> => {
    const response = await apiClient.get<ApiResponse<ThreadData>>(
      `/telegram/actionable-items/${itemId}/thread`
    );
    return response.data;
  },

  /**
   * Get current execution session for an actionable item
   * GET /telegram/actionable-items/:itemId/session
   */
  getItemSession: async (itemId: string): Promise<ExecutionSession> => {
    const response = await apiClient.get<ApiResponse<ExecutionSession>>(
      `/telegram/actionable-items/${itemId}/session`
    );
    return response.data;
  },

  /**
   * Start execution of an actionable item
   * POST /telegram/actionable-items/:itemId/execute
   */
  executeItem: async (itemId: string): Promise<ExecutionSession> => {
    const response = await apiClient.post<ApiResponse<ExecutionSession>>(
      `/telegram/actionable-items/${itemId}/execute`
    );
    return response.data;
  },

  /**
   * Get execution session status
   * GET /telegram/execution-sessions/:sessionId
   */
  getExecutionSession: async (sessionId: string): Promise<ExecutionSession> => {
    const response = await apiClient.get<ApiResponse<ExecutionSession>>(
      `/telegram/execution-sessions/${sessionId}`
    );
    return response.data;
  },

  /**
   * Confirm or reject pending execution step
   * POST /telegram/execution-sessions/:sessionId/confirm
   */
  confirmExecutionStep: async (
    sessionId: string,
    action: 'confirm' | 'reject',
    data?: unknown
  ): Promise<ExecutionSession> => {
    const response = await apiClient.post<ApiResponse<ExecutionSession>>(
      `/telegram/execution-sessions/${sessionId}/confirm`,
      { action, data }
    );
    return response.data;
  },
};
