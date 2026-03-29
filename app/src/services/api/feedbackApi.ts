import type { ApiResponse } from '../../types/api';
import { apiClient } from '../apiClient';

interface FeedbackItem {
  id: string;
  type: 'bug' | 'feature_request' | 'general';
  title: string;
  description: string;
  steps?: string;
  status: 'open' | 'in_progress' | 'closed';
  createdAt: string;
  updatedAt: string;
  // Add other fields based on backend schema
}

interface CreateFeedbackData {
  type: FeedbackItem['type'];
  title: string;
  description: string;
  steps?: string;
}

interface UpdateFeedbackData {
  title?: string;
  description?: string;
  steps?: string;
}

/**
 * Feedback API endpoints
 */
export const feedbackApi = {
  /**
   * Submit feedback (bug, feature_request, general)
   * POST /feedback
   */
  createFeedback: async (feedback: CreateFeedbackData): Promise<FeedbackItem> => {
    const response = await apiClient.post<ApiResponse<FeedbackItem>>('/feedback', feedback);
    return response.data;
  },

  /**
   * List current user's feedback
   * GET /feedback
   */
  getFeedback: async (): Promise<FeedbackItem[]> => {
    const response = await apiClient.get<ApiResponse<FeedbackItem[]>>('/feedback');
    return response.data;
  },

  /**
   * Get a single feedback item
   * GET /feedback/:id
   */
  getFeedbackById: async (id: string): Promise<FeedbackItem> => {
    const response = await apiClient.get<ApiResponse<FeedbackItem>>(`/feedback/${id}`);
    return response.data;
  },

  /**
   * Update feedback (description, steps, etc.)
   * PUT /feedback/:id
   */
  updateFeedback: async (id: string, updates: UpdateFeedbackData): Promise<FeedbackItem> => {
    const response = await apiClient.put<ApiResponse<FeedbackItem>>(`/feedback/${id}`, updates);
    return response.data;
  },
};
