// https://vitepress.dev/guide/custom-theme
import type { Theme } from "vitepress";
import DefaultTheme from "vitepress/theme";
import CustomLayout from "./layouts/CustomLayout.vue";
import { createPinia } from "pinia";

import "./style.css";
import "uno.css";

export default {
    extends: DefaultTheme,
    Layout: CustomLayout,
    enhanceApp({ app }) {
        app.use(createPinia());
    },
} satisfies Theme;
