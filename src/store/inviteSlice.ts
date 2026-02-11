import { createAsyncThunk, createSlice } from '@reduxjs/toolkit';

import { inviteApi } from '../services/api/inviteApi';
import type { InviteCode } from '../types/invite';

interface InviteState {
  codes: InviteCode[];
  isLoading: boolean;
  error: string | null;
  redeemStatus: 'idle' | 'loading' | 'success' | 'error';
  redeemError: string | null;
}

const initialState: InviteState = {
  codes: [],
  isLoading: false,
  error: null,
  redeemStatus: 'idle',
  redeemError: null,
};

export const fetchInviteCodes = createAsyncThunk(
  'invite/fetchInviteCodes',
  async (_, { rejectWithValue }) => {
    try {
      return await inviteApi.getMyInviteCodes();
    } catch (error) {
      const msg =
        error && typeof error === 'object' && 'error' in error
          ? String(error.error)
          : 'Failed to fetch invite codes';
      return rejectWithValue(msg);
    }
  }
);

export const redeemCode = createAsyncThunk(
  'invite/redeemCode',
  async (code: string, { dispatch, rejectWithValue }) => {
    try {
      const result = await inviteApi.redeemInviteCode(code);
      // Re-fetch codes after successful redeem
      dispatch(fetchInviteCodes());
      return result;
    } catch (error) {
      const msg =
        error && typeof error === 'object' && 'error' in error
          ? String(error.error)
          : 'Failed to redeem invite code';
      return rejectWithValue(msg);
    }
  }
);

const inviteSlice = createSlice({
  name: 'invite',
  initialState,
  reducers: {
    clearRedeemStatus: state => {
      state.redeemStatus = 'idle';
      state.redeemError = null;
    },
  },
  extraReducers: builder => {
    builder
      // fetchInviteCodes
      .addCase(fetchInviteCodes.pending, state => {
        state.isLoading = true;
        state.error = null;
      })
      .addCase(fetchInviteCodes.fulfilled, (state, action) => {
        state.isLoading = false;
        state.codes = action.payload;
      })
      .addCase(fetchInviteCodes.rejected, (state, action) => {
        state.isLoading = false;
        state.error = action.payload as string;
      })
      // redeemCode
      .addCase(redeemCode.pending, state => {
        state.redeemStatus = 'loading';
        state.redeemError = null;
      })
      .addCase(redeemCode.fulfilled, state => {
        state.redeemStatus = 'success';
      })
      .addCase(redeemCode.rejected, (state, action) => {
        state.redeemStatus = 'error';
        state.redeemError = action.payload as string;
      });
  },
});

export const { clearRedeemStatus } = inviteSlice.actions;
export default inviteSlice.reducer;
