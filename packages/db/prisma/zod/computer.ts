import * as z from "zod"
import { CompleteUser, RelatedUserModel } from "./index"

export const ComputerModel = z.object({
  id: z.string(),
  brand: z.string(),
  cores: z.number().int(),
  userId: z.string(),
})

export interface CompleteComputer extends z.infer<typeof ComputerModel> {
  user: CompleteUser
}

/**
 * RelatedComputerModel contains all relations on your model in addition to the scalars
 *
 * NOTE: Lazy required in case of potential circular dependencies within schema
 */
export const RelatedComputerModel: z.ZodSchema<CompleteComputer> = z.lazy(() => ComputerModel.extend({
  user: RelatedUserModel,
}))
