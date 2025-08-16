---
title: Playground
sidebar: false
editLink: false
lastUpdated: false
layout: home
---

<script setup lang="ts">
import Playground from "../.vitepress/theme/components/Playground.vue";
</script>

<h1 align="center">Playground</h1>

<Suspense>
  <Playground />
  <template #loading>
    <p>Loading...</p>
  </template>
</Suspense>

<style scoped>
</style>
