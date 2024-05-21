/*
  Warnings:

  - Added the required column `userId` to the `Computer` table without a default value. This is not possible if the table is not empty.

*/
-- AlterTable
ALTER TABLE "Computer" ADD COLUMN     "userId" TEXT NOT NULL;

-- CreateIndex
CREATE INDEX "Computer_userId_idx" ON "Computer"("userId");
