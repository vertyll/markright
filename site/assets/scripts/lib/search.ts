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
 * The pages that hold every word of the query, best first: a word in the title counts most, then in the description,
 * then each time it is in the text. At most limit of them.
 */
export function search(entries: readonly SearchEntry[], query: string, limit = 10): SearchHit[] {
  const words = terms(query);
  if (words.length === 0) return [];
  const hits: SearchHit[] = [];
  for (const entry of entries) {
    const title = entry.title.toLowerCase();
    const description = entry.description.toLowerCase();
    const body = entry.body.toLowerCase();
    let score = 0;
    for (const word of words) {
      const inBody = occurrences(body, word);
      if (!title.includes(word) && !description.includes(word) && inBody === 0) {
        score = 0;
        break;
      }
      score += (title === word ? 20 : 0) + (title.includes(word) ? 10 : 0) + (description.includes(word) ? 4 : 0);
      score += Math.min(inBody, 10);
    }
    if (score > 0) hits.push({ entry, score, excerpt: excerpt(entry.body, words[0] ?? "") });
  }
  return hits.sort((a, b) => b.score - a.score).slice(0, limit);
}
