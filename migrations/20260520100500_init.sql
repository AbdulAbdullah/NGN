 create extension if not exists pgcrypto;
 
 -- Users (auth)
 create table if not exists users (
   id uuid primary key default gen_random_uuid(),
   email text not null unique,
   display_name text not null,
   password_hash text not null,
   role text not null default 'user',
   created_at timestamptz not null default now()
 );
 
 -- Products + identifiers
 create table if not exists products (
   id uuid primary key default gen_random_uuid(),
   name text null,
   brand text null,
   category text not null default 'UNKNOWN',
   community_status text not null default 'UNKNOWN',
   created_at timestamptz not null default now()
 );
 
 create table if not exists product_identifiers (
   id uuid primary key default gen_random_uuid(),
   product_id uuid not null references products(id) on delete cascade,
   kind text not null, -- GTIN | NAFDAC_NO | QR_TEXT
   value text not null,
   created_at timestamptz not null default now(),
   unique(kind, value)
 );
 
 -- Community claims
 create table if not exists claims (
   id uuid primary key default gen_random_uuid(),
   product_id uuid not null references products(id) on delete cascade,
   user_id uuid not null references users(id) on delete cascade,
   status text not null, -- LEGIT | SUSPECT
   note text null,
   created_at timestamptz not null default now()
 );
 
 create index if not exists idx_claims_product_id on claims(product_id);
 create index if not exists idx_claims_user_id on claims(user_id);
 
