import { ref } from 'vue';

const show = ref(false);
const message = ref('');
const type = ref<'success' | 'error' | 'info' | 'warn'>('success');

export function useToast() {
  function showToast(msg: string, t: 'success' | 'error' | 'info' | 'warn') {
    message.value = msg;
    type.value = t;
    show.value = true;
  }
  function closeToast() {
    show.value = false;
  }
  return { show, message, type, showToast, closeToast };
}
