-- Phase 06-04 publish snapshot event schema update.
--
-- Run this against the production Oracle catalog schema before deploying a
-- controller image that records included edit IDs for successful publishes.
-- The script is safe to re-run: it creates the publish snapshot table and
-- supporting edit-event index only when they are absent.

declare
  table_count number;
begin
  select count(*)
    into table_count
    from user_tables
   where table_name = 'AUTOGRAPH_PUBLISH_JOB_EVENTS';

  if table_count = 0 then
    execute immediate q'[
      create table autograph_publish_job_events (
        publish_job_id varchar2(36) not null,
        edit_event_id varchar2(36) not null,
        created_at timestamp default current_timestamp not null,
        constraint autograph_publish_job_events_pk
          primary key (publish_job_id, edit_event_id),
        constraint autograph_publish_job_events_job_fk
          foreign key (publish_job_id) references autograph_publish_jobs(id) on delete cascade,
        constraint autograph_publish_job_events_event_fk
          foreign key (edit_event_id) references autograph_edit_events(id) on delete cascade
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
   where index_name = 'AUTOGRAPH_PUBLISH_JOB_EVENTS_EVENT_IDX';

  if index_count = 0 then
    execute immediate q'[
      create index autograph_publish_job_events_event_idx
        on autograph_publish_job_events(edit_event_id)
    ]';
  end if;
end;
/

commit;
