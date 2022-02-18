CREATE TABLE "main"(
    "_link" TEXT,
    "id" NUMERIC,
    "title" TEXT,
    "releasedate" TIMESTAMP,
    "rating_code" TEXT,
    "rating_name" TEXT);

CREATE TABLE "developer"(
    "_link" TEXT,
    "_link_main" TEXT,
    "name" TEXT);

CREATE TABLE "platforms"(
    "_link" TEXT,
    "_link_main" TEXT,
    "name" TEXT);

