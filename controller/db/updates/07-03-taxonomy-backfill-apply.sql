-- Phase 7 taxonomy backfill apply script.
-- Review every statement before applying to live Oracle.
-- Mapping source: .planning/phases/07-metadata-taxonomy-and-public-facets/taxonomy-backfill-mapping.json

begin
  -- legacy value: Tr
  update autograph_items set format = 'Trading Card' where id = '11111111-1111-4111-8111-111111111111';
  -- legacy value: custom
  update autograph_items set origin = 'Custom' where id = '11111111-1111-4111-8111-111111111111';
  -- legacy value: Japanese
  update autograph_items set language = 'Japanese' where id = '11111111-1111-4111-8111-111111111111';
  -- legacy value: Star Wars
  insert into autograph_item_franchises (item_id, franchise, sort_order)
    select '11111111-1111-4111-8111-111111111111', 'Star Wars', 0 from dual
    where not exists (select 1 from autograph_item_franchises where item_id = '11111111-1111-4111-8111-111111111111' and franchise = 'Star Wars');
  -- legacy value: Young Jedi
  update autograph_items set product_line = 'Young Jedi' where id = '11111111-1111-4111-8111-111111111111';
  -- legacy value: actor
  update autograph_item_signers set item_role = 'actor' where item_id = '11111111-1111-4111-8111-111111111111' and item_role is null;
  -- legacy value: Tra
  update autograph_items set format = 'Trading Card' where id = '22222222-2222-4222-8222-222222222222';
  -- legacy value: Star Wars CCG
  update autograph_items set product_line = 'Star Wars CCG' where id = '22222222-2222-4222-8222-222222222222';
  -- legacy value: voice actor
  update autograph_item_signers set item_role = 'voice actor' where item_id = '22222222-2222-4222-8222-222222222222' and item_role is null;
  -- legacy value: Trading Card
  update autograph_items set format = 'Trading Card' where id = '33333333-3333-4333-8333-333333333333';
end;
/
