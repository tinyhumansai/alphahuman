import type { ApiResponse } from '../../types/api';
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
import { injectAll } from '../../lib/ai/injector';
import type { Message } from '../../lib/ai/providers/interface';

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

  /** POST /chat/sendMessage — send a user message with SOUL + TOOLS injection */
  sendMessage: async (
    message: string,
    conversationId: string,
    options: { injectSoul?: boolean } = { injectSoul: true }
  ): Promise<SendMessageResponseData> => {
    let processedMessage = message;

    if (options.injectSoul) {
      try {
        const userMessage: Message = {
          role: 'user',
          content: [{ type: 'text', text: message }]
        };

        const injectedMessage = await injectAll(userMessage, {
          mode: 'context-block',
          includeMetadata: false
        });

        // Extract the processed text
        const textContent = injectedMessage.content
          .filter(block => block.type === 'text')
          .map(block => (block as { text: string }).text)
          .join('\n');

        processedMessage = textContent;
        console.log('✅ SOUL + TOOLS injection successful in threadApi sendMessage');
      } catch (error) {
        // Graceful degradation - log error but continue with original message
        console.warn('⚠️ SOUL + TOOLS injection failed in threadApi sendMessage:', error);
      }
    }

    const response = await apiClient.post<ApiResponse<SendMessageResponseData>>(
      '/chat/sendMessage',
      { message: processedMessage, conversationId }
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
