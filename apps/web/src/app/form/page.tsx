"use client";
import React from "react";
import { Button } from "@repo/ui/components";
import { useForm } from "react-hook-form";
import { InsertComputer, insertComputerParams } from "@repo/api/src/computers";
import { toast } from "sonner";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation } from "@tanstack/react-query";
import { createComputerAction } from "@/src/lib/api/computer";

export default function Page() {
  const createComputer = useMutation({
    mutationKey: ["createComputer"],
    mutationFn: createComputerAction,
    onError(e) {
      toast.error((e as Error).message);
    },
    onSuccess(d) {
      toast.success("Computer Created Successfully");
    },
  });
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<Omit<InsertComputer, "userId">>({
    resolver: zodResolver(insertComputerParams.omit({ userId: true })),
  });
  return (
    <form
      className="w-full h-screen items-center justify-center flex flex-col gap-4 "
      onSubmit={handleSubmit((data) => {
        createComputer.mutate({ brand: data.brand, cores: data.cores });
      })}
    >
      <div>
        <label>Brand</label>
        <input type="text" {...register("brand")} required />
        <div>{errors.brand?.message}</div>
      </div>
      <div>
        <label>Cores</label>
        <input
          type="number"
          {...register("cores", { valueAsNumber: true })}
          required
        />
        <div>{errors.cores?.message}</div>
      </div>
      <Button
        isLoading={createComputer.isLoading}
        disabled={createComputer.isLoading}
      >
        Create Computer
      </Button>
    </form>
  );
}
