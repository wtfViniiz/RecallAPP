import { describe, it, expect } from 'vitest';
import { escapeHtml, sanitizeUrl, renderMarkdown, htmlToMarkdown } from '../utils.js';

describe('escapeHtml', () => {
  it('escapes HTML special characters', () => {
    expect(escapeHtml('<script>alert(1)</script>')).toBe('&lt;script&gt;alert(1)&lt;/script&gt;');
  });
  it('escapes quotes and ampersands', () => {
    expect(escapeHtml('a & b "c" \'d\'')).toBe('a &amp; b &quot;c&quot; &#39;d&#39;');
  });
  it('returns empty string for empty input', () => {
    expect(escapeHtml('')).toBe('');
  });
  it('handles null/undefined gracefully', () => {
    expect(escapeHtml(null)).toBe('');
    expect(escapeHtml(undefined)).toBe('');
  });
});

describe('sanitizeUrl', () => {
  it('allows http/https URLs', () => {
    expect(sanitizeUrl('https://example.com')).toBe('https://example.com');
  });
  it('blocks javascript: protocol', () => {
    expect(sanitizeUrl('javascript:alert(1)')).toBe('#blocked');
  });
  it('blocks data: protocol', () => {
    expect(sanitizeUrl('data:text/html,<script>alert(1)</script>')).toBe('#blocked');
  });
  it('blocks protocol-relative URLs', () => {
    expect(sanitizeUrl('//evil.com/script')).toBe('#blocked');
  });
  it('returns input unchanged for safe URLs', () => {
    expect(sanitizeUrl('http://example.com')).toBe('http://example.com');
    expect(sanitizeUrl('https://example.com/path?q=1')).toBe('https://example.com/path?q=1');
  });
});

describe('renderMarkdown', () => {
  it('renders bold text', () => {
    const result = renderMarkdown('**bold**');
    expect(result).toContain('<strong>bold</strong>');
  });
  it('renders italic text', () => {
    const result = renderMarkdown('*italic*');
    expect(result).toContain('<em>italic</em>');
  });
  it('renders inline code', () => {
    const result = renderMarkdown('`code`');
    expect(result).toContain('<code>code</code>');
  });
  it('escapes HTML in content', () => {
    const result = renderMarkdown('<script>alert(1)</script>');
    expect(result).not.toContain('<script>');
    expect(result).toContain('&lt;script&gt;');
  });
  it('renders code blocks', () => {
    const result = renderMarkdown('```\ncode\n```');
    expect(result).toContain('<pre>');
    expect(result).toContain('code');
  });
  it('renders blockquotes', () => {
    const result = renderMarkdown('> quote');
    expect(result).toContain('<blockquote>');
  });
  it('renders unordered lists', () => {
    const result = renderMarkdown('- item 1\n- item 2');
    expect(result).toContain('<ul>');
  });
  it('sanitizes javascript: links', () => {
    const result = renderMarkdown('[click](javascript:alert(1))');
    expect(result).not.toContain('javascript:');
    expect(result).toContain('#blocked');
  });
  it('returns empty string for empty input', () => {
    expect(renderMarkdown('')).toBe('');
    expect(renderMarkdown(null)).toBe('');
    expect(renderMarkdown(undefined)).toBe('');
  });
});

describe('htmlToMarkdown', () => {
  it('converts bold HTML to markdown', () => {
    const result = htmlToMarkdown('<strong>bold</strong>');
    expect(result).toContain('**bold**');
  });
  it('converts italic HTML to markdown', () => {
    const result = htmlToMarkdown('<em>italic</em>');
    expect(result).toContain('*italic*');
  });
  it('strips script tags', () => {
    const result = htmlToMarkdown('<script>alert(1)</script>text');
    expect(result).not.toContain('<script>');
    expect(result).toContain('text');
  });
  it('returns empty string for empty input', () => {
    expect(htmlToMarkdown('')).toBe('');
    expect(htmlToMarkdown(null)).toBe('');
  });
});
