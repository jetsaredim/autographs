-- Phase 08-01 private image adjustment metadata schema update.
--
-- Run this against the production Oracle catalog schema before deploying a
-- controller image that persists non-destructive image adjustments.
-- The script is safe to re-run: it adds the nullable private metadata column
-- only when it is absent.

declare
  column_count number;
begin
  select count(*)
    into column_count
    from user_tab_columns
   where table_name = 'AUTOGRAPH_IMAGES'
     and column_name = 'ADJUSTMENT_JSON';

  if column_count = 0 then
    execute immediate
      'alter table autograph_images add adjustment_json clob';
  end if;
end;
/

commit;
