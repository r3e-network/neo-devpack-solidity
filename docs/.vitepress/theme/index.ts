import DefaultTheme from 'vitepress/theme';
import type { Theme } from 'vitepress';
import './custom.css';
import StandardsMirror from './components/StandardsMirror.vue';
import StandardEntry from './components/StandardEntry.vue';

const theme: Theme = {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component('StandardsMirror', StandardsMirror);
    app.component('StandardEntry', StandardEntry);
  }
};

export default theme;
