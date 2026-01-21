// Utility functions for the sample project

export function deprecatedHelper(): void {
  console.log("This function is deprecated and should be removed");
}

export function formatDate(date: Date): string {
  return date.toISOString();
}

export function deprecatedLogger(message: string): void {
  console.log(`[DEPRECATED] ${message}`);
}

export function calculateSum(a: number, b: number): number {
  return a + b;
}

export const DEPRECATED_CONSTANT = "remove me";

export const APP_VERSION = "1.0.0";
