"use client";

import { Button, Input, Label } from "@repo/ui/components";
import { signIn } from "@repo/auth/react";
import React, { useState } from "react";

export function SignInForm() {
  const [email, setEmail] = useState("");
  const [loading, setLoading] = useState(false);
  return (
    <>
      <div className="grid gap-2 sm:gap-4">
        <Label htmlFor="email">Email</Label>
        <Input
          onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
            setEmail(e.target.value);
          }}
          className="py-5"
          id="email"
          type="email"
          placeholder="m@example.com"
        />
      </div>
      <Button
        disabled={loading}
        onClick={async () => {
          setLoading(true);
          await signIn("email", { email, callbackUrl: "/" });
          setLoading(false);
        }}
        className={`w-full py-5 ${loading && "cursor-not-allowed"}`}
      >
        Continue With Email
      </Button>
    </>
  );
}
