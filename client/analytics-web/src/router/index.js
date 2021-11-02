import Vue from 'vue'
import VueRouter from 'vue-router'
import Home from '../views/Home.vue'
import Log from '../views/Log.vue'
import Timeline from '../views/Timeline.vue'

Vue.use(VueRouter)

const routes = [
  {
    path: '/',
    name: 'Home',
    component: Home
  },
  {
    path: '/log',
    name: 'Log',
    component: Log
  },
  {
    path: '/timeline/:process_id',
    name: 'Timeline',
    component: Timeline
  },
  {
    path: '/about',
    name: 'About',
    // route level code-splitting
    // this generates a separate chunk (about.[hash].js) for this route
    // which is lazy-loaded when the route is visited.
    component: function () {
      return import('../views/About.vue')
    }
  }
]

const router = new VueRouter({
  routes
})

export default router
