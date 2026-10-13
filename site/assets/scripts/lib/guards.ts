/** Whether value is an object whose fields can be read by name: what JSON or another module hands over. */
export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
