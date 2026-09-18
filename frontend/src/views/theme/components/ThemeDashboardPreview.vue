<script setup lang="ts">
import type { DashboardTrendKind } from '@/api/modules/dashboard'
import { shallowRef } from 'vue'

import DashboardContent from '@/views/dashboard/components/DashboardContent.vue'
import {
  dashboardSnapshotView,
  dashboardTrendView,
} from '@/views/dashboard/composables/useDashboard'

import { themeDashboardSummary } from '../fixtures/dashboard'

const snapshot = dashboardSnapshotView(themeDashboardSummary)
const trend = dashboardTrendView(themeDashboardSummary.trend)
const trendKind = shallowRef<DashboardTrendKind>('usage')
const searchQuery = shallowRef('')
const providerQuery = shallowRef('')
</script>

<template>
  <DashboardContent
    v-model:trend-kind="trendKind"
    v-model:search-query="searchQuery"
    v-model:provider-query="providerQuery"
    last-refreshed-at="刚刚更新"
    :metrics="snapshot.metrics"
    :trend-points="trend.points"
    :trend-summary="trend.summary"
    :health-timeline="snapshot.healthTimeline"
    :account-usage="snapshot.accountUsage"
    :wire-profiles="snapshot.wireProfiles"
    :usage-records="snapshot.usageRecords"
    :pool-summary="snapshot.poolSummary"
    :capacity-info="snapshot.capacityInfo"
    :rotation-strategy="snapshot.rotationStrategy"
  />
</template>
