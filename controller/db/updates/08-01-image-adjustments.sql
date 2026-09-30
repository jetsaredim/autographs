-- Phase 08-01 private image adjustment metadata schema update.
--
-- Run this against the production Oracle catalog schema before deploying a
-- controller image that persists non-destructive image adjustments.
-- The script is safe to re-run: it adds the nullable private metadata column
-- only when it is absent, then safely converges the edit-event constraint to
-- the Phase 08 value set without dropping the last enabled constraint first.

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
  canonical_constraint_count number;
  canonical_new_count number;
  temporary_constraint_count number;
  temporary_new_count number;
begin
  select count(*)
    into canonical_constraint_count
    from user_constraints
   where table_name = 'AUTOGRAPH_EDIT_EVENTS'
     and constraint_name = 'AUTOGRAPH_EDIT_EVENTS_TYPE_CK';

  select count(*)
    into canonical_new_count
    from user_constraints
   where table_name = 'AUTOGRAPH_EDIT_EVENTS'
     and constraint_name = 'AUTOGRAPH_EDIT_EVENTS_TYPE_CK'
     and constraint_type = 'C'
     and status = 'ENABLED'
     and search_condition_vc like '%''created''%'
     and search_condition_vc like '%''metadataUpdated''%'
     and search_condition_vc like '%''imageAdded''%'
     and search_condition_vc like '%''imageRemoved''%'
     and search_condition_vc like '%''imageReplaced''%'
     and search_condition_vc like '%''imageAdjustmentChanged''%'
     and search_condition_vc like '%''primaryImageChanged''%'
     and search_condition_vc like '%''publicationChanged''%'
     and search_condition_vc like '%''cleanupChanged''%';

  select count(*)
    into temporary_constraint_count
    from user_constraints
   where table_name = 'AUTOGRAPH_EDIT_EVENTS'
     and constraint_name = 'AUTOGRAPH_EDIT_EVENTS_TYPE_V08';

  select count(*)
    into temporary_new_count
    from user_constraints
   where table_name = 'AUTOGRAPH_EDIT_EVENTS'
     and constraint_name = 'AUTOGRAPH_EDIT_EVENTS_TYPE_V08'
     and constraint_type = 'C'
     and status = 'ENABLED'
     and search_condition_vc like '%''created''%'
     and search_condition_vc like '%''metadataUpdated''%'
     and search_condition_vc like '%''imageAdded''%'
     and search_condition_vc like '%''imageRemoved''%'
     and search_condition_vc like '%''imageReplaced''%'
     and search_condition_vc like '%''imageAdjustmentChanged''%'
     and search_condition_vc like '%''primaryImageChanged''%'
     and search_condition_vc like '%''publicationChanged''%'
     and search_condition_vc like '%''cleanupChanged''%';

  if canonical_new_count > 0 then
    if temporary_constraint_count > 0 then
      execute immediate
        'alter table autograph_edit_events drop constraint autograph_edit_events_type_v08';
    end if;
  else
    if temporary_new_count = 0 then
      if temporary_constraint_count > 0 then
        execute immediate
          'alter table autograph_edit_events drop constraint autograph_edit_events_type_v08';
      end if;

      execute immediate q'[
        alter table autograph_edit_events add constraint autograph_edit_events_type_v08
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
    end if;

    if canonical_constraint_count > 0 then
      execute immediate
        'alter table autograph_edit_events drop constraint autograph_edit_events_type_ck';
    end if;

    execute immediate
      'alter table autograph_edit_events rename constraint autograph_edit_events_type_v08 to autograph_edit_events_type_ck';
  end if;
end;
/

commit;
