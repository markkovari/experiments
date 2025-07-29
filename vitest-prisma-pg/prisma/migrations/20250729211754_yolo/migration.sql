-- CreateTable
CREATE TABLE "public"."User" (
    "id" INTEGER NOT NULL,
    "name" TEXT,
    "age" INTEGER NOT NULL DEFAULT 18,

    CONSTRAINT "User_pkey" PRIMARY KEY ("id")
);
