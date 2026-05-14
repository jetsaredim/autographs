create table autograph_schema_migrations (
  version varchar2(32) primary key,
  name varchar2(255) not null,
  applied_at timestamp default current_timestamp not null
)

create table autograph_items (
  id varchar2(36) primary key,
  title varchar2(255) not null,
  signer varchar2(255) not null,
  description clob,
  category varchar2(100) not null,
  object_reference varchar2(512),
  event_name varchar2(255),
  event_location varchar2(255),
  source varchar2(255),
  inscription varchar2(1000),
  certification_company varchar2(255),
  certification_id varchar2(255),
  estimated_year number(4),
  publication_status varchar2(24) default 'draft' not null,
  created_at timestamp default current_timestamp not null,
  updated_at timestamp default current_timestamp not null,
  constraint autograph_items_publication_ck
    check (publication_status in ('draft', 'published', 'archived'))
)

create table autograph_item_tags (
  item_id varchar2(36) not null,
  tag varchar2(80) not null,
  created_at timestamp default current_timestamp not null,
  constraint autograph_item_tags_pk primary key (item_id, tag),
  constraint autograph_item_tags_item_fk
    foreign key (item_id) references autograph_items(id) on delete cascade
)

create table autograph_images (
  id varchar2(36) primary key,
  item_id varchar2(36) not null,
  storage_namespace varchar2(128) not null,
  bucket_name varchar2(255) not null,
  object_key varchar2(1024) not null,
  content_type varchar2(255) not null,
  byte_size number(19),
  checksum varchar2(255),
  etag varchar2(255),
  is_primary char(1) default 'N' not null,
  sort_order number(10) default 0 not null,
  alt_text varchar2(500),
  created_at timestamp default current_timestamp not null,
  updated_at timestamp default current_timestamp not null,
  primary_item_id generated always as (
    case when is_primary = 'Y' then item_id end
  ) virtual,
  constraint autograph_images_item_fk
    foreign key (item_id) references autograph_items(id) on delete cascade,
  constraint autograph_images_primary_ck
    check (is_primary in ('Y', 'N')),
  constraint autograph_images_object_uq
    unique (storage_namespace, bucket_name, object_key),
  constraint autograph_images_one_primary_uq
    unique (primary_item_id)
)

create index autograph_items_signer_idx on autograph_items(signer)

create index autograph_items_category_idx on autograph_items(category)

create index autograph_items_publication_idx on autograph_items(publication_status)

create index autograph_images_item_order_idx on autograph_images(item_id, sort_order)
