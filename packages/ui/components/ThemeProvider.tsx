"use client";

import * as React from "react";
import { ThemeProvider as NextThemesProvider } from "next-themes";
import { Provider } from "@radix-ui/react-tooltip";
import { type ThemeProviderProps } from "next-themes/dist/types";

interface ThemeProviderWithChildrenProps extends ThemeProviderProps {
  children: React.ReactNode;
}

export function ThemeProvider({
  children,
  ...props
}: ThemeProviderWithChildrenProps) {
  return (
    <NextThemesProvider {...props}>
      <Provider>{children}</Provider>
    </NextThemesProvider>
  );
}
