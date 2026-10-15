export function getErrorMessage(error: unknown): string | null {
  return error instanceof Error ? error.message : null;
}
