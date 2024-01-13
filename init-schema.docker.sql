-- This file is almost the same as `scrapy.sql` but contains only the necessary to initialize the database once.
-- also, it doesn't hardcode the database name or user so the values are free to be defined in `docker-compose.yml`

CREATE EXTENSION unaccent;

CREATE TABLE "seller" (
  "seller_id" integer PRIMARY KEY,
  "seller_name" text,
  "seller_url" text,
  "seller_nb_announces" integer,
  "seller_is_pro" boolean
);

CREATE TABLE "okkazeo_announce" (
  "oa_id" integer UNIQUE NOT NULL,
  "oa_last_modification_date" timestamptz NOT NULL,
  "oa_name" text NOT NULL,
  "oa_image" text NOT NULL,
  "oa_price" real NOT NULL,
  "oa_url" text NOT NULL,
  "oa_extension" text,
  "oa_seller" integer REFERENCES seller("seller_id"),
  "oa_barcode" bigint,
  "oa_city" text,
  "oa_nbr_player" integer
);

CREATE TABLE "deal" (
  "deal_id" SERIAL PRIMARY KEY,
  "deal_oa_id" integer REFERENCES okkazeo_announce("oa_id") ON DELETE CASCADE,
  "deal_price" integer,
  "deal_percentage" integer
);


CREATE TABLE "reference" (
  "ref_id" SERIAL PRIMARY KEY,
  "ref_oa_id" integer REFERENCES okkazeo_announce("oa_id") ON DELETE CASCADE,
  "ref_name" text,
  "ref_price" real,
  "ref_url" text,
  "ref_available" boolean
);

CREATE TABLE "reviewer" (
  "reviewer_id" SERIAL PRIMARY KEY,
  "reviewer_oa_id" integer REFERENCES okkazeo_announce("oa_id") ON DELETE CASCADE,
  "reviewer_name" text,
  "reviewer_url" text,
  "reviewer_note" real,
  "reviewer_number" integer
);

CREATE TABLE "shipping" (
  "ship_id" SERIAL PRIMARY KEY,
  "ship_oa_id" integer REFERENCES okkazeo_announce("oa_id") ON DELETE CASCADE,
  "ship_shipper" text,
  "ship_price" real
);


CREATE INDEX idx_deal_oa_id ON deal (deal_oa_id);
CREATE INDEX idx_oa_id ON okkazeo_announce (oa_id);
CREATE INDEX idx_reference_oa_id ON reference (ref_oa_id);
CREATE INDEX idx_reviewer_oa_id ON reviewer (reviewer_oa_id);
CREATE INDEX idx_ship_oa_id ON shipping (ship_oa_id);
CREATE INDEX idx_seller ON seller (seller_id);
