import { createAsyncThunk, createSlice } from '@reduxjs/toolkit';

import { threadApi } from '../services/api/threadApi';
import type { Thread, ThreadMessage } from '../types/thread';

interface ThreadState {
  threads: Thread[];
  isLoading: boolean;
  error: string | null;
  selectedThreadId: string | null;
  messages: ThreadMessage[];
  isLoadingMessages: boolean;
  messagesError: string | null;
  createStatus: 'idle' | 'loading' | 'success' | 'error';
  deleteStatus: 'idle' | 'loading' | 'success' | 'error';
  deleteError: string | null;
  purgeStatus: 'idle' | 'loading' | 'success' | 'error';
  sendStatus: 'idle' | 'loading' | 'success' | 'error';
  sendError: string | null;
  panelWidth: number;
  /** threadId -> timestamp when user last viewed that thread (for unread indicators) */
  lastViewedAt: Record<string, number>;
  /** Suggested starter questions for empty threads (from GET /chat/autocomplete) */
  suggestedQuestions: Array<{ text: string; confidence: number }>;
  isLoadingSuggestions: boolean;
  suggestError: string | null;
}

const initialState: ThreadState = {
  threads: [],
  isLoading: false,
  error: null,
  selectedThreadId: null,
  messages: [],
  isLoadingMessages: false,
  messagesError: null,
  createStatus: 'idle',
  deleteStatus: 'idle',
  deleteError: null,
  purgeStatus: 'idle',
  sendStatus: 'idle',
  sendError: null,
  panelWidth: 320,
  lastViewedAt: {},
  suggestedQuestions: [],
  isLoadingSuggestions: false,
  suggestError: null,
};

export const fetchThreads = createAsyncThunk(
  'thread/fetchThreads',
  async (_, { rejectWithValue }) => {
    try {
      const data = await threadApi.getThreads();
      return data.threads;
    } catch (error) {
      const msg =
        error && typeof error === 'object' && 'error' in error
          ? String(error.error)
          : 'Failed to fetch threads';
      return rejectWithValue(msg);
    }
  }
);

export const fetchThreadMessages = createAsyncThunk(
  'thread/fetchThreadMessages',
  async (threadId: string, { rejectWithValue }) => {
    try {
      const data = await threadApi.getThreadMessages(threadId);
      return data.messages;
    } catch (error) {
      const msg =
        error && typeof error === 'object' && 'error' in error
          ? String(error.error)
          : 'Failed to fetch messages';
      return rejectWithValue(msg);
    }
  }
);

export const createThread = createAsyncThunk(
  'thread/createThread',
  async (chatId: number | undefined, { dispatch, rejectWithValue }) => {
    try {
      const data = await threadApi.createThread(chatId);
      dispatch(fetchThreads());
      return data;
    } catch (error) {
      const msg =
        error && typeof error === 'object' && 'error' in error
          ? String(error.error)
          : 'Failed to create thread';
      return rejectWithValue(msg);
    }
  }
);

export const deleteThread = createAsyncThunk(
  'thread/deleteThread',
  async (threadId: string, { dispatch, getState, rejectWithValue }) => {
    try {
      await threadApi.deleteThread(threadId);
      const state = (getState() as { thread: ThreadState }).thread;
      if (state.selectedThreadId === threadId) {
        dispatch(clearSelectedThread());
      }
      return threadId;
    } catch (error) {
      const msg =
        error && typeof error === 'object' && 'error' in error
          ? String(error.error)
          : 'Failed to delete thread';
      return rejectWithValue(msg);
    }
  }
);

export const purgeThreads = createAsyncThunk(
  'thread/purgeThreads',
  async (_, { dispatch, rejectWithValue }) => {
    try {
      const data = await threadApi.purge({
        messages: false,
        agentThreads: true,
        deleteEverything: true,
      });
      dispatch(fetchThreads());
      return data;
    } catch (error) {
      const msg =
        error && typeof error === 'object' && 'error' in error
          ? String(error.error)
          : 'Failed to purge threads';
      return rejectWithValue(msg);
    }
  }
);

export const sendMessage = createAsyncThunk(
  'thread/sendMessage',
  async (
    { threadId, message }: { threadId: string; message: string },
    { dispatch, rejectWithValue }
  ) => {
    try {
      const data = await threadApi.sendMessage(message, threadId);
      // Re-fetch messages to get the stored user message + agent response
      dispatch(fetchThreadMessages(threadId));
      // Re-fetch threads to update lastMessageAt / messageCount in the list
      dispatch(fetchThreads());
      return data;
    } catch (error) {
      const msg =
        error && typeof error === 'object' && 'error' in error
          ? String(error.error)
          : 'Failed to send message';
      return rejectWithValue(msg);
    }
  }
);

export const fetchSuggestedQuestions = createAsyncThunk(
  'thread/fetchSuggestedQuestions',
  async (conversationId: string | undefined, { rejectWithValue }) => {
    try {
      const data = await threadApi.getSuggestQuestions(conversationId);
      return data.suggestions;
    } catch (error) {
      const msg =
        error && typeof error === 'object' && 'error' in error
          ? String(error.error)
          : 'Failed to load suggestions';
      return rejectWithValue(msg);
    }
  }
);

const threadSlice = createSlice({
  name: 'thread',
  initialState,
  reducers: {
    setSelectedThread: (state, action: { payload: string }) => {
      state.selectedThreadId = action.payload;
      state.messages = [];
      state.messagesError = null;
      state.suggestedQuestions = [];
      state.suggestError = null;
    },
    clearSelectedThread: state => {
      state.selectedThreadId = null;
      state.messages = [];
      state.messagesError = null;
      state.suggestedQuestions = [];
      state.suggestError = null;
    },
    clearSuggestedQuestions: state => {
      state.suggestedQuestions = [];
      state.suggestError = null;
    },
    clearCreateStatus: state => {
      state.createStatus = 'idle';
    },
    clearDeleteStatus: state => {
      state.deleteStatus = 'idle';
      state.deleteError = null;
    },
    clearPurgeStatus: state => {
      state.purgeStatus = 'idle';
    },
    addOptimisticMessage: (state, action: { payload: { content: string } }) => {
      state.messages.push({
        id: `optimistic-${Date.now()}`,
        content: action.payload.content,
        type: 'text',
        extraMetadata: {},
        sender: 'user',
        createdAt: new Date().toISOString(),
      });
    },
    removeOptimisticMessages: state => {
      state.messages = state.messages.filter(m => !m.id.startsWith('optimistic-'));
    },
    clearSendError: state => {
      state.sendError = null;
    },
    setPanelWidth: (state, action: { payload: number }) => {
      state.panelWidth = action.payload;
    },
    setLastViewed: (state, action: { payload: string }) => {
      const ts = Date.now();
      state.lastViewedAt[action.payload] = ts;
    },
  },
  extraReducers: builder => {
    builder
      // fetchThreads — only show skeleton on initial load (stale-while-revalidate)
      .addCase(fetchThreads.pending, state => {
        if (state.threads.length === 0) {
          state.isLoading = true;
        }
        state.error = null;
      })
      .addCase(fetchThreads.fulfilled, (state, action) => {
        state.isLoading = false;
        state.threads = action.payload;
      })
      .addCase(fetchThreads.rejected, (state, action) => {
        state.isLoading = false;
        state.error = action.payload as string;
      })
      // fetchThreadMessages
      .addCase(fetchThreadMessages.pending, state => {
        state.isLoadingMessages = true;
        state.messagesError = null;
      })
      .addCase(fetchThreadMessages.fulfilled, (state, action) => {
        state.isLoadingMessages = false;
        state.messages = action.payload;
        // Hide suggestions once thread has messages
        if (action.payload.length > 0) {
          state.suggestedQuestions = [];
          state.suggestError = null;
        }
      })
      .addCase(fetchThreadMessages.rejected, (state, action) => {
        state.isLoadingMessages = false;
        state.messagesError = action.payload as string;
      })
      // createThread
      .addCase(createThread.pending, state => {
        state.createStatus = 'loading';
      })
      .addCase(createThread.fulfilled, (state, action) => {
        state.createStatus = 'success';
        state.selectedThreadId = action.payload.id;
        state.messages = [];
        state.messagesError = null;
      })
      .addCase(createThread.rejected, state => {
        state.createStatus = 'error';
      })
      // deleteThread
      .addCase(deleteThread.pending, state => {
        state.deleteStatus = 'loading';
        state.deleteError = null;
      })
      .addCase(deleteThread.fulfilled, (state, action) => {
        state.deleteStatus = 'success';
        state.threads = state.threads.filter(t => t.id !== action.payload);
      })
      .addCase(deleteThread.rejected, (state, action) => {
        state.deleteStatus = 'error';
        state.deleteError = action.payload as string;
      })
      // purgeThreads
      .addCase(purgeThreads.pending, state => {
        state.purgeStatus = 'loading';
      })
      .addCase(purgeThreads.fulfilled, state => {
        state.purgeStatus = 'success';
        state.selectedThreadId = null;
        state.messages = [];
      })
      .addCase(purgeThreads.rejected, state => {
        state.purgeStatus = 'error';
      })
      // sendMessage
      .addCase(sendMessage.pending, state => {
        state.sendStatus = 'loading';
        state.sendError = null;
      })
      .addCase(sendMessage.fulfilled, state => {
        state.sendStatus = 'success';
        state.suggestedQuestions = [];
        state.suggestError = null;
      })
      .addCase(sendMessage.rejected, (state, action) => {
        state.sendStatus = 'error';
        state.sendError = action.payload as string;
        // Remove optimistic messages so the user doesn't see phantom messages
        state.messages = state.messages.filter(m => !m.id.startsWith('optimistic-'));
      })
      // fetchSuggestedQuestions
      .addCase(fetchSuggestedQuestions.pending, state => {
        state.isLoadingSuggestions = true;
        state.suggestError = null;
      })
      .addCase(fetchSuggestedQuestions.fulfilled, (state, action) => {
        state.isLoadingSuggestions = false;
        state.suggestedQuestions = action.payload;
      })
      .addCase(fetchSuggestedQuestions.rejected, (state, action) => {
        state.isLoadingSuggestions = false;
        state.suggestError = action.payload as string;
        state.suggestedQuestions = [];
      });
  },
});

export const {
  setSelectedThread,
  clearSelectedThread,
  clearCreateStatus,
  clearDeleteStatus,
  clearPurgeStatus,
  addOptimisticMessage,
  removeOptimisticMessages,
  clearSendError,
  clearSuggestedQuestions,
  setPanelWidth,
  setLastViewed,
} = threadSlice.actions;
export default threadSlice.reducer;
