import type { ActionableItem, TimeGroup } from '../../types/intelligence';

/**
 * Groups actionable items by time periods (Today, Yesterday, This Week, Older)
 */
export function groupItemsByTime(items: ActionableItem[]): TimeGroup[] {
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const yesterday = new Date(today.getTime() - 24 * 60 * 60 * 1000);
  const oneWeekAgo = new Date(today.getTime() - 7 * 24 * 60 * 60 * 1000);

  const groups: Record<string, ActionableItem[]> = {
    today: [],
    yesterday: [],
    thisWeek: [],
    older: [],
  };

  items.forEach(item => {
    const itemDate = new Date(item.createdAt);
    const itemDateOnly = new Date(itemDate.getFullYear(), itemDate.getMonth(), itemDate.getDate());

    if (itemDateOnly >= today) {
      groups.today.push(item);
    } else if (itemDateOnly >= yesterday) {
      groups.yesterday.push(item);
    } else if (itemDateOnly >= oneWeekAgo) {
      groups.thisWeek.push(item);
    } else {
      groups.older.push(item);
    }
  });

  // Sort items within each group by priority and then by date (newest first)
  const sortItems = (items: ActionableItem[]) => {
    const priorityOrder = { critical: 0, important: 1, normal: 2 };
    return items.sort((a, b) => {
      const priorityDiff = priorityOrder[a.priority] - priorityOrder[b.priority];
      if (priorityDiff !== 0) return priorityDiff;
      return new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime();
    });
  };

  const timeGroups: TimeGroup[] = [];

  if (groups.today.length > 0) {
    timeGroups.push({ label: 'Today', items: sortItems(groups.today), count: groups.today.length });
  }

  if (groups.yesterday.length > 0) {
    timeGroups.push({
      label: 'Yesterday',
      items: sortItems(groups.yesterday),
      count: groups.yesterday.length,
    });
  }

  if (groups.thisWeek.length > 0) {
    timeGroups.push({
      label: 'This Week',
      items: sortItems(groups.thisWeek),
      count: groups.thisWeek.length,
    });
  }

  if (groups.older.length > 0) {
    timeGroups.push({ label: 'Older', items: sortItems(groups.older), count: groups.older.length });
  }

  return timeGroups;
}

/**
 * Filters items based on various criteria
 */
export function filterItems(
  items: ActionableItem[],
  options: { source?: string; priority?: string; status?: string; searchTerm?: string }
): ActionableItem[] {
  let filtered = [...items];

  if (options.source && options.source !== 'all') {
    filtered = filtered.filter(item => item.source === options.source);
  }

  if (options.priority && options.priority !== 'all') {
    filtered = filtered.filter(item => item.priority === options.priority);
  }

  if (options.status && options.status !== 'all') {
    filtered = filtered.filter(item => item.status === options.status);
  }

  if (options.searchTerm) {
    const term = options.searchTerm.toLowerCase();
    filtered = filtered.filter(
      item =>
        item.title.toLowerCase().includes(term) ||
        item.description?.toLowerCase().includes(term) ||
        item.sourceLabel?.toLowerCase().includes(term)
    );
  }

  return filtered;
}

/**
 * Gets summary statistics for actionable items
 */
export function getItemStats(items: ActionableItem[]) {
  const total = items.length;
  const byPriority = items.reduce(
    (acc, item) => {
      acc[item.priority]++;
      return acc;
    },
    { critical: 0, important: 0, normal: 0 }
  );

  const bySource = items.reduce(
    (acc, item) => {
      acc[item.source] = (acc[item.source] || 0) + 1;
      return acc;
    },
    {} as Record<string, number>
  );

  const newItems = items.filter(item => {
    const diff = Date.now() - item.createdAt.getTime();
    return diff < 5 * 60 * 1000; // Less than 5 minutes
  }).length;

  const expiringSoon = items.filter(item => {
    if (!item.expiresAt) return false;
    const diff = item.expiresAt.getTime() - Date.now();
    return diff < 24 * 60 * 60 * 1000 && diff > 0; // Expires within 24 hours
  }).length;

  return { total, byPriority, bySource, newItems, expiringSoon };
}
