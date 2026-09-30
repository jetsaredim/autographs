-- Phase 08-01 private image adjustment metadata schema update.
--
-- Run this against the production Oracle catalog schema before deploying a
-- controller image that persists non-destructive image adjustments.
-- The script is safe to re-run: it adds the nullable private metadata column
-- only when it is absent, then replaces the edit-event constraint with the
-- Phase 08 value set.

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

declare
  constraint_count number;
begin
  select count(*)
    into constraint_count
    from user_constraints
   where table_name = 'AUTOGRAPH_EDIT_EVENTS'
     and constraint_name = 'AUTOGRAPH_EDIT_EVENTS_TYPE_CK';

  if constraint_count > 0 then
    execute immediate
      'alter table autograph_edit_events drop constraint autograph_edit_events_type_ck';
  end if;

  execute immediate q'[
    alter table autograph_edit_events add constraint autograph_edit_events_type_ck
      check (event_type in (
        'created',
        'metadataUpdated',
        'imageAdded',
        'imageRemoved',
        'imageReplaced',
        'imageAdjustmentChanged',
        'primaryImageChanged',
        'publicationChanged',
        'cleanupChanged'
      ))
  ]';
end;
/

commit;
