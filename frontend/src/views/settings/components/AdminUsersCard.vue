<script setup lang="ts">
import type { AdminRole, AdminUser } from '@/api'
import { UserPlus } from '@lucide/vue'
import { onMounted, ref } from 'vue'
import { createAdminUser as createAdminUserApi, getAdminUsers } from '@/api'
import BaseButton from '@/components/base/BaseButton.vue'
import BaseCard from '@/components/base/BaseCard.vue'
import BaseInput from '@/components/base/BaseInput.vue'
import BaseSelect from '@/components/base/BaseSelect.vue'

withDefaults(defineProps<{
  readOnly?: boolean
}>(), {
  readOnly: false,
})

const users = ref<AdminUser[]>([])
const loading = ref(false)
const submitting = ref(false)
const error = ref('')
const username = ref('')
const password = ref('')
const role = ref<AdminRole>('readonly')

const roleOptions = [
  { label: '只读管理员', value: 'readonly', description: '可查看数据，不能修改配置' },
  { label: '完整管理员', value: 'admin', description: '可查看并修改管理配置' },
]

function errorMessage(value: unknown, fallback: string): string {
  return value instanceof Error && value.message ? value.message : fallback
}

async function loadUsers() {
  loading.value = true
  error.value = ''
  try {
    users.value = await getAdminUsers()
  }
  catch (value) {
    error.value = errorMessage(value, '管理员账户加载失败')
  }
  finally {
    loading.value = false
  }
}

async function submit() {
  if (submitting.value || !username.value.trim() || !password.value)
    return

  submitting.value = true
  error.value = ''
  try {
    const created = await createAdminUserApi({
      username: username.value.trim(),
      password: password.value,
      role: role.value,
    })
    users.value = [...users.value, created]
    username.value = ''
    password.value = ''
    role.value = 'readonly'
  }
  catch (value) {
    error.value = errorMessage(value, '管理员账户创建失败')
  }
  finally {
    submitting.value = false
  }
}

onMounted(() => {
  void loadUsers()
})
</script>

<template>
  <BaseCard title="管理员账户" description="创建独立的完整管理员或只读管理员账户">
    <template #actions>
      <span v-if="readOnly" class="rounded-cp-sm bg-cp-fill-tertiary px-2.5 py-1 text-cp-xs font-bold text-cp-text-secondary">
        当前账户只读
      </span>
    </template>

    <div class="grid gap-4">
      <p v-if="error" role="alert" class="m-0 text-cp-sm text-cp-error-text">
        {{ error }}
      </p>

      <div v-if="loading" role="status" class="rounded-cp bg-cp-fill-quaternary px-4 py-3 text-cp-sm text-cp-text-tertiary">
        正在加载管理员账户…
      </div>
      <div v-else class="overflow-x-auto rounded-cp bg-cp-fill-quaternary">
        <table class="w-full min-w-120 text-left text-cp-sm">
          <thead class="text-cp-xs font-bold text-cp-text-tertiary">
            <tr>
              <th class="px-4 py-3">
                用户名
              </th>
              <th class="px-4 py-3">
                权限
              </th>
              <th class="px-4 py-3">
                创建时间
              </th>
            </tr>
          </thead>
          <tbody class="text-cp-text">
            <tr v-for="user in users" :key="user.username" class="border-t border-cp-split">
              <td class="px-4 py-3 font-mono font-bold">
                {{ user.username }}
              </td>
              <td class="px-4 py-3">
                {{ user.role === 'readonly' ? '只读管理员' : '完整管理员' }}
              </td>
              <td class="px-4 py-3 font-mono text-cp-text-secondary">
                {{ new Date(user.createdAt).toLocaleString() }}
              </td>
            </tr>
            <tr v-if="users.length === 0">
              <td colspan="3" class="px-4 py-4 text-center text-cp-text-tertiary">
                暂无管理员账户
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <form v-if="!readOnly" class="grid gap-3 border-t border-cp-split pt-4 sm:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_minmax(0,1fr)_auto]" @submit.prevent="submit">
        <BaseInput v-model="username" aria-label="管理员用户名" placeholder="用户名" autocomplete="off" />
        <BaseInput v-model="password" aria-label="管理员密码" placeholder="至少 12 位密码" type="password" autocomplete="new-password" />
        <BaseSelect v-model="role" aria-label="管理员权限" :options="roleOptions" />
        <BaseButton type="submit" variant="primary" :loading="submitting" :disabled="!username.trim() || !password">
          <template #icon>
            <UserPlus class="size-4" />
          </template>
          创建账户
        </BaseButton>
      </form>
    </div>
  </BaseCard>
</template>
