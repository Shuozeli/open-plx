import { createContext, useContext } from "react";

interface ThemeContextValue {
  dark: boolean;
}

export const ThemeContext = createContext<ThemeContextValue>({ dark: false });

/** Returns true if dark mode is active. */
export function useDarkMode(): boolean {
  return useContext(ThemeContext).dark;
}
