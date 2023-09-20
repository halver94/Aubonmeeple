-- Suppression des tables si elles existent déjà
DROP TABLE IF EXISTS game;
DROP TABLE IF EXISTS deal;
DROP TABLE IF EXISTS reviewer;
DROP TABLE IF EXISTS reference;
DROP TABLE IF EXISTS shipping;
DROP TABLE IF EXISTS ship;
DROP TABLE IF EXISTS seller;
DROP TABLE IF EXISTS okkazeo_announce;
DROP DATABASE IF EXISTS scraper;
DROP USER IF EXISTS scrapy;

-- Création de la base de données
CREATE DATABASE scraper;

-- Création de l'utilisateur "scrapy"
CREATE USER scrapy WITH PASSWORD 'scrapyscrapy';
GRANT ALL PRIVILEGES ON DATABASE scraper TO scrapy;

-- Connexion à la base de données
\c scraper;

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

-- Assignation des privilèges sur les tables
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO scrapy;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO scrapy;
