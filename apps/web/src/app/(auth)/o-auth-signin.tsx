"use client";

import React, { useState } from "react";
import { toast } from "sonner";
import { signIn } from "@repo/auth/react";
import type { OAuthProviders } from "@repo/auth/server";
import { Button } from "@repo/ui/components";
import { Icons } from "@repo/ui/icons";
import type { LucideProps } from "@repo/ui/icons";

type OAuthProviderProps = {
  name: string;
  provider: OAuthProviders;
  icon: keyof typeof Icons;
}[];
const oauthprovider: OAuthProviderProps = [
  { name: "Google", provider: "google", icon: "google" },
  { name: "Github", provider: "github", icon: "github" },
  { name: "Discord", provider: "discord", icon: "discord" },
];

const OAuthSignIn = () => {
  const [isLoading, setIsLoading] = useState<boolean>(false);
  async function handleClick(provider: OAuthProviders) {
    try {
      const data = await signIn(provider, { callbackUrl: "/", redirect: true });
      if (data?.error) {
        console.log(data.error);
        toast.error(data.error);
      } else {
        // data && data.url && router.push(data?.url);
      }
    } catch (error) {
      console.log(error);
    }
  }

  return (
    <div className="grid grid-cols-1 gap-2 sm:grid-cols-1 sm:gap-3">
      {oauthprovider.map((provider) => {
        const Icon = Icons[provider.icon] as ({
          ...props
        }: LucideProps) => React.JSX.Element;
        return (
          <Button
            aria-label={`Sign in with ${provider.name}`}
            key={provider.provider}
            variant="outline"
            className="w-full bg-background sm:w-auto py-5"
            onClick={async () => {
              setIsLoading(true);
              await handleClick(provider.provider);
            }}
            disabled={isLoading}
          >
            {isLoading ? (
              <Icons.spinner
                className="mr-2 h-4 w-4 animate-spin"
                aria-label="loading..."
              />
            ) : (
              <Icon className="mr-2 h-4 w-4" aria-label="loading..." />
            )}
            Continue with {provider.name}
          </Button>
        );
      })}
    </div>
  );
};

export default OAuthSignIn;
