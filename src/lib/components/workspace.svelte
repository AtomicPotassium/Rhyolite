<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import TabsStore from "$lib/stores/tabs.store";
  import tabService from "$lib/services/tab.service";
  import { type Tab, TabType } from "$lib/types/tab";
  import Document from "./document.svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";

  let tabs: Tab[] = $state([]);
  let currentTab: Tab | null = $state(null);

  onMount(() => {
    // Listen for the 'Tabs' event from the backend
    const tabslisten = listen<Tab[]>("Tabs", (event) => {
      tabs = event.payload;
    });
    const currentTablisten = listen<Tab>("Current_Tab", (event) => {
      currentTab = event.payload;
    });
    return () => {
      tabslisten.then((unsub) => unsub());
      currentTablisten.then((unsub) => unsub());
    };
  });

  const onOpenTab = (tab: Tab) => {
    tabService.switchTab(tab);
  };
</script>

<div class="flex grow justify-center mt-[30px] px-10 overflow-auto">
  <!-- {#each tabs as tab} -->
  <!-- {#if tab.tabType === TabType.Document || tab.tabType === undefined} -->
  <!-- TODO: Q: How to switch between tabs? -->
  <!-- 1: Have all tabs as separate DOM Elements, set display:none on inactive tabs -->
  <!--    Pro: possibly retained DOM states. Con: Too large DOM-->
  <!-- 2: Have only active tab in DOM -->
  <!--    Pro: possibly retained DOM states. Con: Too large DOM-->
  <Document />
  <!-- {/if} -->
  <!-- {/each} -->
</div>
