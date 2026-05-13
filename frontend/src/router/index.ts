import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import Lobby from '../components/Lobby.vue'
import GameBoard from '../components/GameBoard.vue'

// Typisierung des Route-Arrays!
const routes: Array<RouteRecordRaw> = [
  {
    path: '/',
    name: 'Lobby',
    component: Lobby
  },
  {
    path: '/game/:id',
    name: 'Game',
    component: GameBoard,
    props: true 
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router