import { isRecord } from "./guards.ts";

/** One issue in the playground's document. */
export interface PlaygroundIssue {
  readonly line: number;
  readonly source: string;
  readonly message: string;
}

/** What markright makes of a document: its issues, the report `markright check` prints and the formatted document. */
export interface Outcome {
  readonly issues: readonly PlaygroundIssue[];
  readonly report: string;
  readonly formatted: string;
}

/** The syntaxes a configuration is written in, and the file each is read from. */
export type Syntax = "yaml" | "toml";

/** The file a configuration in syntax is read from. */
export const configFile = (syntax: Syntax): string => `.markright.${syntax}`;

/** Whether value names a syntax. */
export const isSyntax = (value: string): value is Syntax => value === "yaml" || value === "toml";

/** Whether value is an issue: its line, the rule that found it and what it says. */
function isIssue(value: unknown): value is PlaygroundIssue {
  return (
    isRecord(value) &&
    typeof value["line"] === "number" &&
    typeof value["source"] === "string" &&
    typeof value["message"] === "string"
  );
}

/** Reads what the module returns, and fails on anything that is not an outcome. */
export function parseOutcome(value: unknown): Outcome {
  if (!isRecord(value)) throw new TypeError("markright returned no outcome");
  const { issues, report, formatted } = value;
  if (!Array.isArray(issues) || !issues.every(isIssue) || typeof report !== "string" || typeof formatted !== "string") {
    throw new TypeError("markright returned a malformed outcome");
  }
  return { issues, report, formatted };
}

/** What a thrown value says: the message of an error, or the value as text. */
export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

/** The last line of the report: how many issues there are. */
export function summary(outcome: Outcome): string {
  return outcome.report.trimEnd().split("\n").at(-1) ?? "";
}

/** Where line, counted from 1, starts and ends in text; past the end, the end of the text. */
export function lineRange(text: string, line: number): [start: number, end: number] {
  let start = 0;
  for (let current = 1; current < line; current++) {
    const next = text.indexOf("\n", start);
    if (next === -1) return [text.length, text.length];
    start = next + 1;
  }
  const end = text.indexOf("\n", start);
  return [start, end === -1 ? text.length : end];
}

/** Calls run once the calls have stopped for wait milliseconds. */
export function debounce<A extends unknown[]>(run: (...args: A) => void, wait: number): (...args: A) => void {
  let timer: ReturnType<typeof setTimeout> | undefined;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => run(...args), wait);
  };
}

/**
 * The tab a key moves to among count tabs, from the one at index, as the WAI-ARIA tabs pattern sets out: the arrows
 * move to the next or previous tab and wrap around, Home and End to the first and the last. Any other key moves
 * nowhere.
 */
export function tabAfterKey(key: string, index: number, count: number): number | undefined {
  switch (key) {
    case "ArrowRight":
      return (index + 1) % count;
    case "ArrowLeft":
      return (index - 1 + count) % count;
    case "Home":
      return 0;
    case "End":
      return count - 1;
    default:
      return undefined;
  }
}
