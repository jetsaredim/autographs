alter table autograph_images add (
  original_filename varchar2(512)
)

create table autograph_publish_jobs (
  id varchar2(36) primary key,
  publish_mode varchar2(24) not null,
  status varchar2(24) not null,
  release_id varchar2(128),
  error_detail clob,
  started_at timestamp default current_timestamp not null,
  finished_at timestamp,
  created_at timestamp default current_timestamp not null,
  constraint autograph_publish_jobs_mode_ck
    check (publish_mode in ('incremental', 'full')),
  constraint autograph_publish_jobs_status_ck
    check (status in ('queued', 'running', 'succeeded', 'failed'))
)

create table autograph_public_derivatives (
  id varchar2(36) primary key,
  image_id varchar2(36) not null,
  release_id varchar2(128) not null,
  variant_name varchar2(32) not null,
  public_path varchar2(1024) not null,
  content_type varchar2(255) not null,
  byte_size number(19) not null,
  created_at timestamp default current_timestamp not null,
  constraint autograph_public_derivatives_image_fk
    foreign key (image_id) references autograph_images(id) on delete cascade,
  constraint autograph_public_derivatives_variant_ck
    check (variant_name in ('thumbnail', 'detail')),
  constraint autograph_public_derivatives_path_uq
    unique (release_id, public_path)
)

create index autograph_publish_jobs_status_idx
  on autograph_publish_jobs(status, started_at)

create index autograph_public_derivatives_release_idx
  on autograph_public_derivatives(release_id)

