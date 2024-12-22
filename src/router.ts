import { createRouter, createWebHashHistory } from 'vue-router';
import NewCalibration from './pages/NewCalibration.vue';
import PastCalibrations from './pages/PastCalibrations.vue';

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      redirect: '/new'
    },
    {
      path: '/new',
      component: NewCalibration
    },
    {
      path: '/past',
      component: PastCalibrations
    }
  ]
});

export default router;
