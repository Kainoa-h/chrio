// Run in a separate process so camera tests' browser mocks cannot affect Vue DOM setup.
import { Window } from 'happy-dom';
import assert from 'node:assert/strict';

const browser = new Window({ url: 'http://localhost/' });
for (const name of ['window', 'document', 'navigator', 'HTMLElement', 'Element', 'Node', 'NodeFilter', 'Document', 'DocumentFragment', 'SVGElement', 'Event', 'KeyboardEvent', 'PointerEvent', 'MouseEvent', 'CustomEvent', 'MutationObserver', 'ResizeObserver', 'HTMLInputElement', 'HTMLSelectElement', 'HTMLOptionElement']) {
  Object.defineProperty(globalThis, name, { configurable: true, value: (browser as any)[name] });
}
globalThis.getComputedStyle = browser.getComputedStyle.bind(browser) as typeof getComputedStyle;
globalThis.requestAnimationFrame = browser.requestAnimationFrame.bind(browser);
globalThis.cancelAnimationFrame = browser.cancelAnimationFrame.bind(browser);
const { createApp, h, ref, nextTick } = await import('vue');
const { createPinia } = await import('pinia');
const { default: CompareSessionColumn } = await import('../../src/components/CompareSessionColumn.vue');
const sessions = [
  { id: 1, session_number: 1, datetime: '2024-02-29T09:00:00' },
  { id: 2, session_number: 2, datetime: '2025-01-10T09:30:00' },
  { id: 3, session_number: 3, datetime: '2025-01-10T15:00:00' },
  { id: 4, session_number: 4, datetime: '2025-01-10T18:00:00' },
  { id: 5, session_number: 5, datetime: '2025-03-04T10:00:00' },
  { id: 6, session_number: 6, datetime: '2026-08-22T10:00:00' },
];
const selected = ref(sessions[2]);
const olderLimit = ref<number | null>(null);
const newerLimit = ref<number | null>(6);
const host = document.createElement('div');
document.body.append(host);
const app = createApp({ render: () => h(CompareSessionColumn, {
  session: selected.value, allSessions: sessions, olderLimit: olderLimit.value, newerLimit: newerLimit.value,
  images: {}, canRemove: false, isSelectMode: false, selectedKeys: [],
  'onUpdate:session': (session: typeof sessions[number]) => { selected.value = session; },
}) });
app.use(createPinia());
app.mount(host);
const settle = async () => { await nextTick(); await new Promise(r => setTimeout(r, 70)); await nextTick(); };
const calendarButton = () => host.querySelector<HTMLButtonElement>('[aria-label="Choose session by date"]')!;
const dropdown = () => host.querySelector<HTMLButtonElement>('[aria-label="Session to compare"]')!;
const popup = () => document.querySelector<HTMLElement>('[aria-label="Choose a session date"]');
const button = (label: string) => popup()!.querySelector<HTMLButtonElement>(`[aria-label="${label}"]`)!;
const click = async (element: HTMLElement) => { assert(element); element.click(); await settle(); };
const day = (date: string) => popup()!.querySelector<HTMLElement>(`[data-reka-calendar-cell-trigger][data-value="${date}"]`)!;
const byText = (text: string) => [...popup()!.querySelectorAll<HTMLButtonElement>('button')].find(b => b.textContent?.trim() === text)!;
const key = async (element: HTMLElement, value: string, code = value) => {
  element.dispatchEvent(new KeyboardEvent('keydown', { key: value, code, bubbles: true, cancelable: true }));
  await settle();
};
await settle();
await click(calendarButton());
assert.match(button('Choose month').textContent!, /January/);
assert.equal(day('2025-01-10').getAttribute('aria-disabled'), null);
assert.match(day('2025-01-10').className, /bg-blue-50/);
assert.equal(day('2025-01-11').getAttribute('aria-disabled'), 'true');
await click(day('2025-01-11'));
assert.equal(selected.value.id, 3, 'Unavailable dates cannot select a session');
await click(day('2025-01-10'));
assert.equal(selected.value.id, 3, 'Multiple sessions require an explicit choice');
const choices = [...popup()!.querySelectorAll<HTMLButtonElement>('button')].filter(b => b.textContent?.includes('Session #'));
assert.equal(choices.length, 3);
assert.match(choices[0].textContent!, /09:30/);
assert.match(choices[1].textContent!, /15:00/);
assert.match(choices[2].textContent!, /18:00/);
await click(choices[0]);
assert.equal(selected.value.id, 2);
assert.equal(popup(), null);
assert.match(dropdown().textContent!, /#2/);
// Reopening and choosing another session on the already-selected date must work too.
await click(calendarButton());
await click(day('2025-01-10'));
const repeatedChoices = [...popup()!.querySelectorAll<HTMLButtonElement>('button')].filter(b => b.textContent?.includes('Session #'));
assert.equal(repeatedChoices.length, 3);
await click(repeatedChoices[2]);
assert.equal(selected.value.id, 4);
assert.match(dropdown().textContent!, /#4/);
await click(calendarButton());
await click(button('Choose year'));
assert(byText('2024'));
assert(byText('2025'));
assert(!byText('2026'), 'Column limits exclude unavailable years');
await click(byText('2024'));
assert.equal(byText('Jan').disabled, true);
assert.equal(byText('Feb').disabled, false);
assert.match(byText('Feb').className, /bg-blue-50/);
await click(byText('Feb'));
assert(day('2024-02-29'));
day('2024-02-29').focus();
await key(day('2024-02-29'), 'Enter');
assert.equal(selected.value.id, 1, 'Keyboard selection chooses a single session immediately');
assert.match(dropdown().textContent!, /#1/);
// Selecting through the existing dropdown also updates the calendar's opening month.
await key(dropdown(), 'ArrowDown');
const option = [...document.querySelectorAll<HTMLElement>('[role="option"]')].find(option => option.textContent?.includes('#5'))!;
option.focus();
await key(option, 'Enter');
assert.equal(selected.value.id, 5);
await click(calendarButton());
assert.match(button('Choose month').textContent!, /March/);
await key(button('Close calendar'), 'Escape');
assert.equal(popup(), null);
assert.equal(document.activeElement, calendarButton());
selected.value = sessions[1];
olderLimit.value = 1;
newerLimit.value = 4;
await settle();
await click(calendarButton());
assert.equal(button('Previous month with sessions').disabled, true);
assert.equal(button('Next month with sessions').disabled, true);
await click(button('Choose year'));
assert(!byText('2024'));
assert(byText('2025'));
await click(button('Close calendar'));
olderLimit.value = 9;
await settle();
assert.equal(calendarButton().disabled, true, 'An empty candidate list disables the picker');
app.unmount();
await browser.happyDOM.abort();
console.log('Calendar availability, same-day choices, column limits, keyboard and dropdown synchronization checks passed.');
