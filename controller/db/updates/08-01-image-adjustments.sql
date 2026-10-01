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
  canonical_enabled_count number;
  canonical_new_count number;
  canonical_condition varchar2(4000);
  temporary_constraint_count number;
  temporary_new_count number;
  temporary_condition varchar2(4000);
  expected_condition constant varchar2(4000) := q'~event_typein('created','metadataUpdated','imageAdded','imageRemoved','imageReplaced','imageAdjustmentChanged','primaryImageChanged','publicationChanged','cleanupChanged')~';

  function normalize_event_condition(condition varchar2) return varchar2 is
    normalized varchar2(4000) := '';
    current_character varchar2(1);
    position pls_integer := 1;
    inside_literal boolean := false;
  begin
    if condition is null then
      return null;
    end if;

    while position <= length(condition) loop
      current_character := substr(condition, position, 1);
      if inside_literal then
        normalized := normalized || current_character;
        if current_character = '''' then
          if position < length(condition)
             and substr(condition, position + 1, 1) = '''' then
            normalized := normalized || '''';
            position := position + 1;
          else
            inside_literal := false;
          end if;
        end if;
      elsif current_character = '''' then
        normalized := normalized || current_character;
        inside_literal := true;
      elsif current_character = '"'
         or current_character in (' ', chr(9), chr(10), chr(13)) then
        null;
      else
        normalized := normalized || lower(current_character);
      end if;
      position := position + 1;
    end loop;

    if inside_literal then
      return null;
    end if;
    return normalized;
  end;
begin
  select count(*)
    into canonical_constraint_count
    from user_constraints
   where table_name = 'AUTOGRAPH_EDIT_EVENTS'
     and constraint_name = 'AUTOGRAPH_EDIT_EVENTS_TYPE_CK';

  select count(*), max(search_condition_vc)
    into canonical_enabled_count, canonical_condition
    from user_constraints
   where table_name = 'AUTOGRAPH_EDIT_EVENTS'
     and constraint_name = 'AUTOGRAPH_EDIT_EVENTS_TYPE_CK'
     and constraint_type = 'C'
     and status = 'ENABLED';

  if canonical_enabled_count = 1
     and normalize_event_condition(canonical_condition) = expected_condition then
    canonical_new_count := 1;
  else
    canonical_new_count := 0;
  end if;

  select count(*)
    into temporary_constraint_count
    from user_constraints
   where table_name = 'AUTOGRAPH_EDIT_EVENTS'
     and constraint_name = 'AUTOGRAPH_EDIT_EVENTS_TYPE_V08';

  select count(*), max(search_condition_vc)
    into temporary_new_count, temporary_condition
    from user_constraints
   where table_name = 'AUTOGRAPH_EDIT_EVENTS'
     and constraint_name = 'AUTOGRAPH_EDIT_EVENTS_TYPE_V08'
     and constraint_type = 'C'
     and status = 'ENABLED';

  if temporary_new_count = 1
     and normalize_event_condition(temporary_condition) = expected_condition then
    temporary_new_count := 1;
  else
    temporary_new_count := 0;
  end if;

  if canonical_new_count > 0 then
    if temporary_constraint_count > 0 then
      execute immediate
        'alter table autograph_edit_events drop constraint autograph_edit_events_type_v08';
    end if;
  else
    if temporary_new_count > 0 then
      if canonical_constraint_count > 0 then
        execute immediate
          'alter table autograph_edit_events drop constraint autograph_edit_events_type_ck';
      end if;
      execute immediate
        'alter table autograph_edit_events rename constraint autograph_edit_events_type_v08 to autograph_edit_events_type_ck';
    elsif canonical_enabled_count > 0 then
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
      execute immediate
        'alter table autograph_edit_events drop constraint autograph_edit_events_type_ck';
      execute immediate
        'alter table autograph_edit_events rename constraint autograph_edit_events_type_v08 to autograph_edit_events_type_ck';
    else
      if canonical_constraint_count > 0 then
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
      if temporary_constraint_count > 0 then
        execute immediate
          'alter table autograph_edit_events drop constraint autograph_edit_events_type_v08';
      end if;
    end if;
  end if;
end;
/

commit;
