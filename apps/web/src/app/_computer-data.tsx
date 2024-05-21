"use client";
import { GetComputerReturns, getComputers } from "@repo/api/src/computers";
import { Button } from "@repo/ui/components";
import { useMutation, useQuery } from "@tanstack/react-query";
import { toast } from "sonner";
import {
  createComputerAction,
  deleteUsersComputersAction,
} from "../lib/api/computer";

export function ComputerData({
  initialData,
}: {
  initialData: GetComputerReturns;
}) {
  const computers = useQuery({
    queryKey: ["getAllComputer"],
    queryFn: async () => {
      return await getComputers();
    },
    initialData,
  });
  const createComp = useMutation({
    mutationKey: ["createComputer"],
    mutationFn: createComputerAction,
    onSuccess: async ({ data, serverError }) => {
      const comps = await computers.refetch();
      if (serverError) {
        return toast.error(serverError);
      }
      return toast.success(
        `Your created a computer, computer id: ${data?.computer.id}, total numbers of computers you hold: ${comps.data?.totalComputer}`
      );
    },
    onError: async (error) => {
      toast.error((error as Error).message);
    },
  });
  const deleteUsersAllComputers = useMutation({
    mutationKey: ["deleteUsersComputers"],
    mutationFn: deleteUsersComputersAction,
    onSuccess: async ({ data }) => {
      console.log(data);
      toast.success(
        `You deleted all your computers, total numbers of computers you deleted: ${data?.computersDeleted}`
      );
      await computers.refetch();
    },
    onError: async (error) => {
      toast.error((error as Error).message);
    },
  });

  return (
    <div className="flex flex-col place-items-center justify-center space-y-4">
      <div className="m-4 space-x-4">
        <Button
          size={"sm"}
          isLoading={createComp.isLoading}
          onClick={() => {
            createComp.mutate({
              brand: "intel",
              cores: 3,
            });
          }}
        >
          Create Computer
        </Button>
        <Button
          isLoading={deleteUsersAllComputers.isLoading}
          variant={"destructive"}
          size={"sm"}
          onClick={() => {
            deleteUsersAllComputers.mutate(undefined);
          }}
        >
          Delete All Computers
        </Button>
      </div>
      <div className="w-full bg-gradient-to-r from-background to-accent border rounded-md p-6">
        <pre>{JSON.stringify(computers.data?.computers, null, 2)}</pre>
      </div>
    </div>
  );
}
