import { createSlice, type PayloadAction } from '@reduxjs/toolkit';

export interface NotionUserProfile {
  id: string;
  name?: string | null;
  email?: string | null;
  type?: string | null;
  avatar_url?: string | null;
}

export interface NotionPageSummary {
  id: string;
  title: string;
  url: string | null;
  last_edited_time: string;
  content_text: string | null;
}

/** AI-generated summary for a Notion page, published by the notion skill. */
export interface NotionSummary {
  id: number;
  pageId: string;
  url: string | null;
  summary: string;
  category: string | null;
  sentiment: string;
  topics: string[];
  sourceCreatedAt: string;
  sourceUpdatedAt: string;
}

interface NotionState {
  /** Profile of the connected Notion user (from Notion skill) */
  profile: NotionUserProfile | null;
  /** Pages fetched after OAuth connection (from Notion skill) */
  pages: NotionPageSummary[];
  /** AI-generated page summaries published by the Notion skill */
  summaries: NotionSummary[];
}

const initialState: NotionState = { profile: null, pages: [], summaries: [] };

const notionSlice = createSlice({
  name: 'notion',
  initialState,
  reducers: {
    setNotionProfile(state, action: PayloadAction<NotionUserProfile | null>) {
      state.profile = action.payload;
    },
    clearNotionProfile(state) {
      state.profile = null;
    },
    setNotionPages(state, action: PayloadAction<NotionPageSummary[]>) {
      state.pages = action.payload;
    },
    clearNotionPages(state) {
      state.pages = [];
    },
    setNotionSummaries(state, action: PayloadAction<NotionSummary[]>) {
      state.summaries = action.payload;
    },
    clearNotionSummaries(state) {
      state.summaries = [];
    },
    clearNotionData(state) {
      state.profile = null;
      state.pages = [];
      state.summaries = [];
    },
  },
});

export const {
  setNotionProfile,
  clearNotionProfile,
  setNotionPages,
  clearNotionPages,
  setNotionSummaries,
  clearNotionSummaries,
  clearNotionData,
} = notionSlice.actions;
export default notionSlice.reducer;
