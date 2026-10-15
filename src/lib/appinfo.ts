// App identity + first-launch helpers (frontend-only, no backend needed).
// The version is stamped from package.json at build time.

import pkg from "../../package.json";

export const APP_NAME = "PlotVale";

export const APP_VERSION: string = (pkg as { version: string }).version;

/** Locale key for a time-of-day greeting (no interpolation needed). */
export function greetingKey(): string {
  const h = new Date().getHours();
  if (h >= 5 && h < 12) return "greet_morning";
  if (h >= 12 && h < 18) return "greet_afternoon";
  if (h >= 18 && h < 24) return "greet_evening";
  return "greet_night";
}

const ONBOARDED_KEY = "plotvale.onboarded.v1";

export function hasSeenOnboarding(): boolean {
  try {
    return localStorage.getItem(ONBOARDED_KEY) === "1";
  } catch {
    return true;
  }
}

export function markOnboarded(): void {
  try {
    localStorage.setItem(ONBOARDED_KEY, "1");
  } catch {
    /* private mode: the tour simply shows again next time */
  }
}
