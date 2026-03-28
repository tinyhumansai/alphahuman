import type { ApiResponse } from '../../types/api';
import type { OutboundRoute } from '../../types/channels';
import type {
  PurgeRequestBody,
  PurgeResultData,
  SendMessageResponseData,
  SuggestQuestionsData,
  ThreadCreateData,
  ThreadDeleteData,
  ThreadMessagesData,
  ThreadsListData,
} from '../../types/thread';
import { apiClient } from '../apiClient';

export const threadApi = {
  /** GET /threads — list all threads for the authenticated user */
  getThreads: async (): Promise<ThreadsListData> => {
    const response = await apiClient.get<ApiResponse<ThreadsListData>>('/threads');
    return response.data;
  },

  /** POST /threads — create a new thread */
  createThread: async (chatId?: number): Promise<ThreadCreateData> => {
    const response = await apiClient.post<ApiResponse<ThreadCreateData>>(
      '/threads',
      chatId != null ? { chatId } : undefined
    );
    return response.data;
  },

  /** GET /threads/:threadId/messages — get messages for a thread */
  getThreadMessages: async (threadId: string): Promise<ThreadMessagesData> => {
    const response = await apiClient.get<ApiResponse<ThreadMessagesData>>(
      `/threads/${encodeURIComponent(threadId)}/messages`
    );
    return response.data;
  },

  /** DELETE /threads/:threadId — delete a single thread */
  deleteThread: async (threadId: string): Promise<ThreadDeleteData> => {
    const response = await apiClient.delete<ApiResponse<ThreadDeleteData>>(
      `/threads/${encodeURIComponent(threadId)}`
    );
    return response.data;
  },

  /** POST /chat/sendMessage — send a user message (context injection done by caller) */
  sendMessage: async (
    message: string,
    conversationId: string,
    route?: OutboundRoute
  ): Promise<SendMessageResponseData> => {
    const response = await apiClient.post<ApiResponse<SendMessageResponseData>>(
      '/chat/sendMessage',
      {
        message,
        conversationId,
        ...(route ? { channel: route.channel, channelAuthMode: route.authMode } : {}),
      }
    );
    return response.data;
  },

  /** GET /chat/autocomplete — suggested starter questions (e.g. for a new/empty thread) */
  getSuggestQuestions: async (conversationId?: string): Promise<SuggestQuestionsData> => {
    const url =
      conversationId != null
        ? `/chat/autocomplete?conversationId=${encodeURIComponent(conversationId)}`
        : '/chat/autocomplete';
    const response = await apiClient.get<ApiResponse<SuggestQuestionsData>>(url);
    return response.data;
  },

  /** POST /purge — purge messages and/or threads */
  purge: async (body: PurgeRequestBody): Promise<PurgeResultData> => {
    const response = await apiClient.post<ApiResponse<PurgeResultData>>('/purge', body);
    return response.data;
  },
};
