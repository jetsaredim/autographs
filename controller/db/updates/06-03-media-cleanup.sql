-- Phase 06-03 media cleanup schema update.
--
-- Run this against the production Oracle catalog schema before deploying a
-- controller image that includes Phase 06-03 cleanup-warning behavior.
-- The script is safe to re-run: it creates the cleanup table and index only
-- when they are absent, then replaces the edit-event type check constraint
-- with the Phase 06-03 value set.

declare
  table_count number;
begin
  select count(*)
    into table_count
    from user_tables
   where table_name = 'AUTOGRAPH_CLEANUP_EVENTS';

  if table_count = 0 then
    execute immediate q'[
      create table autograph_cleanup_events (
        id varchar2(36) primary key,
        item_id varchar2(36) not null,
        image_id varchar2(36) not null,
        operation varchar2(48) not null,
        status varchar2(48) not null,
        admin_message varchar2(500) not null,
        created_at timestamp default current_timestamp not null,
        resolved_at timestamp,
        constraint autograph_cleanup_events_item_fk
          foreign key (item_id) references autograph_items(id) on delete cascade,
        constraint autograph_cleanup_events_status_ck
          check (status in ('succeeded', 'deleteFailed', 'retrySucceeded'))
      )
    ]';
  end if;
end;
/

declare
  index_count number;
begin
  select count(*)
    into index_count
    from user_indexes
   where index_name = 'AUTOGRAPH_CLEANUP_EVENTS_ITEM_STATUS_IDX';

  if index_count = 0 then
    execute immediate q'[
      create index autograph_cleanup_events_item_status_idx
        on autograph_cleanup_events(item_id, status, created_at)
    ]';
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
        'primaryImageChanged',
        'publicationChanged',
        'cleanupChanged'
      ))
  ]';
end;
/

commit;
