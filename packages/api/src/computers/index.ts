import { ComputerModel } from "@repo/db";
import { z } from "zod";

// define your schema and export everything for clients to use
// this is like your interface of your apis
export const insertComputerParams = ComputerModel.omit({ id: true });
export type InsertComputer = z.infer<typeof insertComputerParams>;

export const updateComputerParams = ComputerModel;
export type UpdateComputer = z.infer<typeof updateComputerParams>;

export * from "./mutations";
export * from "./queries";
