"use client";

import { ThemeProvider } from "@repo/ui/components/ThemeProvider";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { PropsWithChildren } from "react";

const Provider = ({ children }: PropsWithChildren) => {
  const client = new QueryClient({
    defaultOptions: { queries: { staleTime: 60000 } },
  });
  return (
    <ThemeProvider attribute="class" defaultTheme="system" enableSystem>
      <QueryClientProvider client={client}>
        {children}
      </QueryClientProvider>
    </ThemeProvider>
  );
};

export default Provider;
