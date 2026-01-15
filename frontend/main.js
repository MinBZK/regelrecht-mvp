// Import @minbzk/storybook web components and CSS
import '@minbzk/storybook';
import '@minbzk/storybook/css';

// Import Vue for the machine action sheet editor
import { createApp } from 'vue';
import App from './src/App.vue';

// Mount Vue app if container exists (only on editor page)
const vueAppContainer = document.getElementById('vue-app');
if (vueAppContainer) {
  const app = createApp(App);
  const vueInstance = app.mount(vueAppContainer);

  // Expose openModal to window for editor buttons
  window.openMachineActionSheet = () => {
    vueInstance.openModal();
  };
}
