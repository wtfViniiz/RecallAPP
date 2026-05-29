import { describe, it, expect } from 'vitest';
import { formatDateTime, formatRelativeDate, formatDate } from '../utils.js';

describe('formatDate', () => {
  it('formats valid ISO date', () => {
    const result = formatDate('2026-05-28T10:30:00Z');
    expect(result).toBeTruthy();
    expect(typeof result).toBe('string');
  });
  it('returns empty for invalid input', () => {
    expect(formatDate('')).toBe('');
    expect(formatDate(null)).toBe('');
  });
});

describe('formatRelativeDate', () => {
  it('returns a string for valid date', () => {
    const now = new Date().toISOString();
    const result = formatRelativeDate(now);
    expect(typeof result).toBe('string');
    expect(result.length).toBeGreaterThan(0);
  });
  it('returns empty for invalid input', () => {
    expect(formatRelativeDate('')).toBe('');
    expect(formatRelativeDate(null)).toBe('');
  });
});

describe('formatDateTime', () => {
  it('formats a valid ISO date string', () => {
    const result = formatDateTime('2026-05-28T10:30:00Z');
    expect(result).toBeTruthy();
    expect(typeof result).toBe('string');
  });
  it('returns empty string for invalid date', () => {
    expect(formatDateTime('not-a-date')).toBe('');
  });
});
