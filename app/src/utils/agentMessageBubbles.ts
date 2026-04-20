/**
 * Split an agent message into render-time bubble segments.
 *
 * Normalize excessive vertical whitespace first, then split only on double
 * newlines. Fenced code blocks stay intact as a single segment so
 * Markdown/code rendering does not fragment unexpectedly.
 * Markdown tables also stay grouped so they can render as dedicated table UI.
 */
export function splitAgentMessageIntoBubbles(content: string): string[] {
  const normalized = content
    .replace(/\r\n/g, '\n')
    .replace(/\n{3,}/g, '\n\n')
    .replace(/\n?\s*<hr\s*\/?>\s*\n?/gi, '\n\n');
  const trimmedContent = normalized.trim();
  if (trimmedContent.length === 0) return [];
  if (!normalized.includes('\n')) {
    return isVisualSeparatorOnly(trimmedContent) ? [] : [trimmedContent];
  }

  const lines = normalized.split('\n');
  const segments: string[] = [];
  let currentLines: string[] = [];
  let inFence = false;

  const flushCurrent = () => {
    const segment = currentLines.join('\n').trim();
    if (segment.length > 0) {
      segments.push(segment);
    }
    currentLines = [];
  };

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    const trimmedLine = line.trim();

    if (!inFence && isMarkdownTableStart(lines, index)) {
      if (currentLines.length > 0) {
        flushCurrent();
      }
      const tableLines = [line, lines[index + 1]];
      index += 2;
      while (index < lines.length && looksLikeMarkdownTableRow(lines[index])) {
        tableLines.push(lines[index]);
        index += 1;
      }
      index -= 1;
      segments.push(tableLines.join('\n').trim());
      continue;
    }

    if (trimmedLine.startsWith('```')) {
      if (!inFence && currentLines.length > 0) {
        flushCurrent();
      }
      currentLines.push(line);
      inFence = !inFence;
      if (!inFence) {
        flushCurrent();
      }
      continue;
    }

    if (inFence) {
      currentLines.push(line);
      continue;
    }

    if (trimmedLine.length === 0) {
      if (currentLines.length > 0) {
        flushCurrent();
      }
      continue;
    }

    currentLines.push(line);
  }

  flushCurrent();
  return segments.filter(segment => !isVisualSeparatorOnly(segment));
}

export interface ParsedMarkdownTable {
  headers: string[];
  rows: string[][];
}

export function parseMarkdownTable(content: string): ParsedMarkdownTable | null {
  const normalized = content.replace(/\r\n/g, '\n').trim();
  const lines = normalized
    .split('\n')
    .map(line => line.trim())
    .filter(line => line.length > 0);

  if (lines.length < 2) return null;
  if (!isMarkdownTableStart(lines, 0)) return null;

  const headers = splitMarkdownTableCells(lines[0]);
  const rows = lines.slice(2).map(splitMarkdownTableCells);

  if (headers.length === 0 || rows.some(row => row.length !== headers.length)) {
    return null;
  }

  return { headers, rows };
}

function isMarkdownTableStart(lines: string[], index: number): boolean {
  const header = lines[index];
  const separator = lines[index + 1];
  if (!header || !separator) return false;
  return looksLikeMarkdownTableRow(header) && looksLikeMarkdownTableSeparator(separator);
}

function looksLikeMarkdownTableRow(line: string): boolean {
  const trimmed = line.trim();
  if (!trimmed.includes('|')) return false;
  const cells = splitMarkdownTableCells(trimmed);
  return cells.length >= 2;
}

function looksLikeMarkdownTableSeparator(line: string): boolean {
  const cells = splitMarkdownTableCells(line);
  if (cells.length < 2) return false;
  return cells.every(cell => /^:?-{3,}:?$/.test(cell));
}

function splitMarkdownTableCells(line: string): string[] {
  return line
    .trim()
    .replace(/^\|/, '')
    .replace(/\|$/, '')
    .split('|')
    .map(cell => cell.trim());
}

function isVisualSeparatorOnly(segment: string): boolean {
  const trimmed = segment.trim();
  if (trimmed.length === 0) return true;
  if (/^<hr\s*\/?>$/i.test(trimmed)) return true;
  return /^(?:-{3,}|\*{3,}|_{3,})$/.test(trimmed.replace(/\s+/g, ''));
}
