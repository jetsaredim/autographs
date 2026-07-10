-- Phase 07-01 taxonomy schema update.
--
-- Run this against the production Oracle catalog schema before deploying a
-- controller image that reads or writes reusable signers, signer credits, and
-- first-class taxonomy fields. The script is additive and intentionally keeps
-- legacy autograph_items.signer, autograph_items.category, and
-- autograph_item_tags available through Phase 7 migration work.

declare
  column_count number;
begin
  select count(*)
    into column_count
    from user_tab_columns
   where table_name = 'AUTOGRAPH_ITEMS'
     and column_name = 'FORMAT';

  if column_count = 0 then
    execute immediate
      q'[alter table autograph_items add format varchar2(80) default 'Trading Card' not null]';
  end if;
end;
/

declare
  column_count number;
begin
  select count(*)
    into column_count
    from user_tab_columns
   where table_name = 'AUTOGRAPH_ITEMS'
     and column_name = 'ORIGIN';

  if column_count = 0 then
    execute immediate
      q'[alter table autograph_items add origin varchar2(24) default 'Official' not null]';
  end if;
end;
/

declare
  column_count number;
begin
  select count(*)
    into column_count
    from user_tab_columns
   where table_name = 'AUTOGRAPH_ITEMS'
     and column_name = 'LANGUAGE';

  if column_count = 0 then
    execute immediate
      q'[alter table autograph_items add language varchar2(40) default 'English' not null]';
  end if;
end;
/

declare
  column_count number;
begin
  select count(*)
    into column_count
    from user_tab_columns
   where table_name = 'AUTOGRAPH_ITEMS'
     and column_name = 'PRODUCT_LINE';

  if column_count = 0 then
    execute immediate
      'alter table autograph_items add product_line varchar2(160)';
  end if;
end;
/

declare
  column_count number;
begin
  select count(*)
    into column_count
    from user_tab_columns
   where table_name = 'AUTOGRAPH_ITEMS'
     and column_name = 'SET_NAME';

  if column_count = 0 then
    execute immediate
      'alter table autograph_items add set_name varchar2(160)';
  end if;
end;
/

declare
  constraint_count number;
begin
  select count(*)
    into constraint_count
    from user_constraints
   where table_name = 'AUTOGRAPH_ITEMS'
     and constraint_name = 'AUTOGRAPH_ITEMS_FORMAT_CK';

  if constraint_count = 0 then
    execute immediate q'[
      alter table autograph_items add constraint autograph_items_format_ck
        check (trim(format) is not null)
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
   where table_name = 'AUTOGRAPH_ITEMS'
     and constraint_name = 'AUTOGRAPH_ITEMS_ORIGIN_CK';

  if constraint_count = 0 then
    execute immediate q'[
      alter table autograph_items add constraint autograph_items_origin_ck
        check (origin in ('Official', 'Custom'))
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
   where table_name = 'AUTOGRAPH_ITEMS'
     and constraint_name = 'AUTOGRAPH_ITEMS_LANGUAGE_CK';

  if constraint_count = 0 then
    execute immediate q'[
      alter table autograph_items add constraint autograph_items_language_ck
        check (language in ('English', 'Japanese', 'Chinese'))
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
   where table_name = 'AUTOGRAPH_SIGNERS'
     and constraint_name = 'AUTOGRAPH_SIGNERS_NORMALIZED_NAME_CK';

  if constraint_count = 0 then
    execute immediate q'[
      alter table autograph_signers add constraint autograph_signers_normalized_name_ck
        check (trim(normalized_name) is not null)
    ]';
  end if;
end;
/

declare
  duplicate_count number;
  constraint_count number;
begin
  select count(*)
    into constraint_count
    from user_constraints
   where table_name = 'AUTOGRAPH_SIGNERS'
     and constraint_name = 'AUTOGRAPH_SIGNERS_NORMALIZED_NAME_UQ';

  if constraint_count = 0 then
    select count(*)
      into duplicate_count
      from (
        select normalized_name
          from autograph_signers
         group by normalized_name
        having count(*) > 1
      );

    if duplicate_count > 0 then
      raise_application_error(
        -20071,
        'Cannot add autograph_signers_normalized_name_uq while duplicate normalized_name values exist.'
      );
    end if;

    execute immediate q'[
      alter table autograph_signers add constraint autograph_signers_normalized_name_uq
        unique (normalized_name)
    ]';
  end if;
end;
/

declare
  table_count number;
begin
  select count(*)
    into table_count
    from user_tables
   where table_name = 'AUTOGRAPH_SIGNERS';

  if table_count = 0 then
    execute immediate q'[
      create table autograph_signers (
        id varchar2(36) primary key,
        display_name varchar2(255) not null,
        normalized_name varchar2(255) not null,
        default_role varchar2(80),
        wikipedia_url varchar2(1000),
        imdb_url varchar2(1000),
        created_at timestamp default current_timestamp not null,
        updated_at timestamp default current_timestamp not null,
        constraint autograph_signers_display_name_ck
          check (trim(display_name) is not null),
        constraint autograph_signers_normalized_name_ck
          check (trim(normalized_name) is not null),
        constraint autograph_signers_normalized_name_uq
          unique (normalized_name)
      )
    ]';
  end if;
end;
/

declare
  table_count number;
begin
  select count(*)
    into table_count
    from user_tables
   where table_name = 'AUTOGRAPH_ITEM_SIGNERS';

  if table_count = 0 then
    execute immediate q'[
      create table autograph_item_signers (
        item_id varchar2(36) not null,
        signer_id varchar2(36) not null,
        sort_order number(10) default 0 not null,
        item_role varchar2(80),
        item_context varchar2(255),
        created_at timestamp default current_timestamp not null,
        constraint autograph_item_signers_pk primary key (item_id, signer_id),
        constraint autograph_item_signers_item_fk
          foreign key (item_id) references autograph_items(id) on delete cascade,
        constraint autograph_item_signers_signer_fk
          foreign key (signer_id) references autograph_signers(id)
      )
    ]';
  end if;
end;
/

declare
  table_count number;
begin
  select count(*)
    into table_count
    from user_tables
   where table_name = 'AUTOGRAPH_ITEM_CHARACTERS';

  if table_count = 0 then
    execute immediate q'[
      create table autograph_item_characters (
        item_id varchar2(36) not null,
        character_name varchar2(160) not null,
        sort_order number(10) default 0 not null,
        created_at timestamp default current_timestamp not null,
        constraint autograph_item_characters_pk primary key (item_id, character_name),
        constraint autograph_item_characters_name_ck
          check (trim(character_name) is not null),
        constraint autograph_item_characters_item_fk
          foreign key (item_id) references autograph_items(id) on delete cascade
      )
    ]';
  end if;
end;
/

declare
  table_count number;
begin
  select count(*)
    into table_count
    from user_tables
   where table_name = 'AUTOGRAPH_ITEM_FRANCHISES';

  if table_count = 0 then
    execute immediate q'[
      create table autograph_item_franchises (
        item_id varchar2(36) not null,
        franchise varchar2(160) not null,
        sort_order number(10) default 0 not null,
        created_at timestamp default current_timestamp not null,
        constraint autograph_item_franchises_pk primary key (item_id, franchise),
        constraint autograph_item_franchises_name_ck
          check (trim(franchise) is not null),
        constraint autograph_item_franchises_item_fk
          foreign key (item_id) references autograph_items(id) on delete cascade
      )
    ]';
  end if;
end;
/

declare
  index_count number;
begin
  select count(*) into index_count from user_indexes
   where index_name = 'AUTOGRAPH_ITEMS_FORMAT_IDX';

  if index_count = 0 then
    execute immediate 'create index autograph_items_format_idx on autograph_items(format)';
  end if;
end;
/

declare
  index_count number;
begin
  select count(*) into index_count from user_indexes
   where index_name = 'AUTOGRAPH_ITEMS_ORIGIN_IDX';

  if index_count = 0 then
    execute immediate 'create index autograph_items_origin_idx on autograph_items(origin)';
  end if;
end;
/

declare
  index_count number;
begin
  select count(*) into index_count from user_indexes
   where index_name = 'AUTOGRAPH_ITEMS_LANGUAGE_IDX';

  if index_count = 0 then
    execute immediate 'create index autograph_items_language_idx on autograph_items(language)';
  end if;
end;
/

declare
  index_count number;
begin
  select count(*) into index_count from user_indexes
   where index_name = 'AUTOGRAPH_ITEMS_PRODUCT_LINE_IDX';

  if index_count = 0 then
    execute immediate
      'create index autograph_items_product_line_idx on autograph_items(product_line)';
  end if;
end;
/

declare
  index_count number;
begin
  select count(*) into index_count from user_indexes
   where index_name = 'AUTOGRAPH_SIGNERS_NORMALIZED_NAME_IDX';

  if index_count = 0 then
    execute immediate
      'create index autograph_signers_normalized_name_idx on autograph_signers(normalized_name)';
  end if;
end;
/

declare
  index_count number;
begin
  select count(*) into index_count from user_indexes
   where index_name = 'AUTOGRAPH_ITEM_SIGNERS_ITEM_ORDER_IDX';

  if index_count = 0 then
    execute immediate
      'create index autograph_item_signers_item_order_idx on autograph_item_signers(item_id, sort_order)';
  end if;
end;
/

declare
  index_count number;
begin
  select count(*) into index_count from user_indexes
   where index_name = 'AUTOGRAPH_ITEM_SIGNERS_SIGNER_IDX';

  if index_count = 0 then
    execute immediate
      'create index autograph_item_signers_signer_idx on autograph_item_signers(signer_id)';
  end if;
end;
/

declare
  index_count number;
begin
  select count(*) into index_count from user_indexes
   where index_name = 'AUTOGRAPH_ITEM_FRANCHISES_VALUE_IDX';

  if index_count = 0 then
    execute immediate
      'create index autograph_item_franchises_value_idx on autograph_item_franchises(franchise)';
  end if;
end;
/

commit;
