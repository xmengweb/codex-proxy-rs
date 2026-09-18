import type { RequestOptions } from '../request'
import request from '../request'

export type AdminRole = 'admin' | 'readonly'

export interface AdminUser {
  username: string
  role: AdminRole
  createdAt: string
}

export function getAdminUsers(options: RequestOptions = {}) {
  return request<AdminUser[]>({
    url: '/api/admin/auth/users',
    method: 'GET',
    ...options,
  })
}

export function createAdminUser(
  data: { username: string, password: string, role: AdminRole },
  options: RequestOptions = {},
) {
  return request<AdminUser>({
    url: '/api/admin/auth/users',
    method: 'POST',
    data,
    ...options,
  })
}
