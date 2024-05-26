"use client";

import { SessionProvider } from "@repo/auth/react";
import { ThemeProvider } from "@repo/ui/components/ThemeProvider";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { PropsWithChildren } from "react";
import { OrderProvider } from "../lib/hooks/order.hooks";

const Provider = ({ children }: PropsWithChildren) => {
  const client = new QueryClient({
    defaultOptions: { queries: { staleTime: 60000 } },
  });
  return (
    <ThemeProvider attribute="class" defaultTheme="system" enableSystem>
      <QueryClientProvider client={client}>
        <OrderProvider>
          <SessionProvider>{children}</SessionProvider>
        </OrderProvider>
      </QueryClientProvider>
    </ThemeProvider>
  );
};

export default Provider;
