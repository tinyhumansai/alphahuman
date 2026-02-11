export interface InviteCodeUser {
  _id: string;
  firstName?: string;
  lastName?: string;
  username?: string;
  telegramId?: string;
}

export interface UsageHistoryEntry {
  userId: InviteCodeUser;
  usedAt: string;
}

export interface InviteCode {
  _id: string;
  code: string;
  owner: string;
  type: 'USER' | 'CAMPAIGN';
  maxUses: number;
  currentUses: number;
  usageHistory: UsageHistoryEntry[];
  isActive: boolean;
  createdAt: string;
}
