-- Phase 07-02 taxonomy backfill review.
-- Mapping source:
-- .planning/phases/07-metadata-taxonomy-and-public-facets/taxonomy-backfill-mapping.json
--
-- Open in SQL Developer before applying the generated PL/SQL script. This file
-- is intentionally read-only and shows legacy category/tag values that need
-- deterministic mapping or human review.

select category as legacy_category, count(*) as item_count
  from autograph_items
 group by category
 order by category;

select tag as legacy_tag, count(*) as item_count
  from autograph_item_tags
 group by tag
 order by tag;

select i.id, i.title, i.signer, i.category, t.tag
  from autograph_items i
  left join autograph_item_tags t on t.item_id = i.id
 where lower(i.title) like '%duplicate%'
    or lower(t.tag) like '%duplicate%'
 order by i.title, t.tag;
