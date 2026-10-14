import { isRecord } from "./guards.ts";

/** A page of the site as the search index holds it. */
export interface SearchEntry {
  readonly url: string;
  readonly title: string;
  readonly description: string;
  readonly body: string;
}

/** A page that matches a query: how well, and the part of its text around the first match. */
export interface SearchHit {
  readonly entry: SearchEntry;
  readonly score: number;
  readonly excerpt: Excerpt;
}

/** A piece of text with the match in its middle, so a page can mark it. */
export interface Excerpt {
  readonly before: string;
  readonly match: string;
  readonly after: string;
}

/** How much text an excerpt keeps on each side of the match. */
const context = 60;

/** Whether value is a page of the index: its address, title, description and text. */
function isEntry(value: unknown): value is SearchEntry {
  return isRecord(value) && ["url", "title", "description", "body"].every((key) => typeof value[key] === "string");
}

/** Reads the index Zola writes, and fails on anything that is not a list of pages. */
export function parseIndex(value: unknown): SearchEntry[] {
  if (!Array.isArray(value) || !value.every(isEntry)) throw new TypeError("malformed search index");
  return value;
}

/** The words of a query, in lower case. */
export function terms(query: string): string[] {
  return query.toLowerCase().split(/\s+/).filter(Boolean);
}

/** How many times term is in text, without overlaps. */
function occurrences(text: string, term: string): number {
  let count = 0;
  for (let at = text.indexOf(term); at !== -1; at = text.indexOf(term, at + term.length)) count++;
  return count;
}

/** The text around the first place term is in body, on one line; the start of body when it is not there. */
export function excerpt(body: string, term: string): Excerpt {
  const text = body.replace(/\s+/g, " ").trim();
  const at = term === "" ? -1 : text.toLowerCase().indexOf(term);
  if (at === -1) return { before: "", match: "", after: text.slice(0, context * 2) };
  const start = Math.max(0, at - context);
  const end = Math.min(text.length, at + term.length + context);
  return {
    before: (start > 0 ? "…" : "") + text.slice(start, at),
    match: text.slice(at, at + term.length),
    after: text.slice(at + term.length, end) + (end < text.length ? "…" : ""),
  };
}

/**
 * How well entry matches words, all in lower case: a word in the title counts most, then in the description, then
 * each time it is in the text, up to ten times. 0 when a word is nowhere.
 */
export function score(entry: SearchEntry, words: readonly string[]): number {
  const title = entry.title.toLowerCase();
  const description = entry.description.toLowerCase();
  const body = entry.body.toLowerCase();
  let total = 0;
  for (const word of words) {
    const inBody = occurrences(body, word);
    const inTitle = title.includes(word);
    const inDescription = description.includes(word);
    if (!inTitle && !inDescription && inBody === 0) return 0;
    if (title === word) total += 20;
    if (inTitle) total += 10;
    if (inDescription) total += 4;
    total += Math.min(inBody, 10);
  }
  return total;
}

/** The pages that hold every word of the query, best first, at most limit of them. */
export function search(entries: readonly SearchEntry[], query: string, limit = 10): SearchHit[] {
  const words = terms(query);
  if (words.length === 0) return [];
  const hits = entries
    .map((entry) => ({ entry, score: score(entry, words) }))
    .filter((hit) => hit.score > 0)
    .map((hit) => ({ ...hit, excerpt: excerpt(hit.entry.body, words[0] ?? "") }));
  return hits.toSorted((a, b) => b.score - a.score).slice(0, limit);
}
