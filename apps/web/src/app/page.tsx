import { ComputerData } from "./_computer-data";
import { Shell } from "@repo/ui/components";
import React, { Suspense } from "react";
import { checkAuth } from "@repo/auth/server";
import { getComputers } from "@repo/api/src/computers";
import { Loader } from "@repo/ui/icons";

export default async function Page() {
  await checkAuth();
  return (
    <Shell
      as={"div"}
      className="flex flex-col place-items-center justify-center"
    >
      <Suspense fallback={<Loader className="animate-spin w-4 h-4" />}>
        <ComputerData initialData={await getComputers()}></ComputerData>
      </Suspense>
    </Shell>
  );
}
