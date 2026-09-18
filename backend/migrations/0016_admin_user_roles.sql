-- 为管理员账户增加显式角色，旧账户默认保持完整管理员权限。
alter table admin_users
    add column role text not null default 'admin';

alter table admin_users
    add constraint admin_users_role_check check (role in ('admin', 'readonly'));
