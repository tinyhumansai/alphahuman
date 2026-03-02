import { createSlice, type PayloadAction } from '@reduxjs/toolkit';

export interface NotionUserProfile {
  id: string;
  name?: string | null;
  email?: string | null;
  type?: string | null;
  avatar_url?: string | null;
}

interface NotionState {
  /** Profile of the connected Notion user (from Notion skill) */
  profile: NotionUserProfile | null;
}

const initialState: NotionState = { profile: null };

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
  },
});

export const { setNotionProfile, clearNotionProfile } = notionSlice.actions;
export default notionSlice.reducer;

