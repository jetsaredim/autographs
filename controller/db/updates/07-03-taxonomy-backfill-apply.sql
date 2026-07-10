-- Phase 7 taxonomy backfill apply script.
-- Review every statement before applying to live Oracle.
-- Mapping source: .planning/phases/07-metadata-taxonomy-and-public-facets/taxonomy-backfill-mapping.json

begin
  merge into autograph_signers signer
    using (select 'e94832b7-db47-5b31-96e1-d54ac4167f67' id, 'Mark Hamill' display_name, 'mark hamill' normalized_name from dual) incoming
    on (signer.normalized_name = incoming.normalized_name)
    when not matched then insert (id, display_name, normalized_name)
      values (incoming.id, incoming.display_name, incoming.normalized_name);
  insert into autograph_item_signers (item_id, signer_id, sort_order, item_role)
    select '11111111-1111-4111-8111-111111111111', signer.id, 0, 'actor' from autograph_signers signer
    where signer.normalized_name = 'mark hamill'
      and not exists (select 1 from autograph_item_signers existing where existing.item_id = '11111111-1111-4111-8111-111111111111' and existing.signer_id = signer.id);
  merge into autograph_signers signer
    using (select '5d3d8eae-2b1b-5008-a7a3-a92fff0aaa2e' id, 'Carrie Fisher' display_name, 'carrie fisher' normalized_name from dual) incoming
    on (signer.normalized_name = incoming.normalized_name)
    when not matched then insert (id, display_name, normalized_name)
      values (incoming.id, incoming.display_name, incoming.normalized_name);
  insert into autograph_item_signers (item_id, signer_id, sort_order, item_role)
    select '22222222-2222-4222-8222-222222222222', signer.id, 0, 'voice actor' from autograph_signers signer
    where signer.normalized_name = 'carrie fisher'
      and not exists (select 1 from autograph_item_signers existing where existing.item_id = '22222222-2222-4222-8222-222222222222' and existing.signer_id = signer.id);
  merge into autograph_signers signer
    using (select '7aa4bcb4-a0f9-5ff1-b43a-d806d00f5a1b' id, 'Unknown Signer' display_name, 'unknown signer' normalized_name from dual) incoming
    on (signer.normalized_name = incoming.normalized_name)
    when not matched then insert (id, display_name, normalized_name)
      values (incoming.id, incoming.display_name, incoming.normalized_name);
  insert into autograph_item_signers (item_id, signer_id, sort_order, item_role)
    select '44444444-4444-4444-8444-444444444444', signer.id, 0, cast(null as varchar2(128)) from autograph_signers signer
    where signer.normalized_name = 'unknown signer'
      and not exists (select 1 from autograph_item_signers existing where existing.item_id = '44444444-4444-4444-8444-444444444444' and existing.signer_id = signer.id);
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
