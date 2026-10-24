<script setup lang="ts">
import { computed, nextTick, ref, shallowRef, watch } from 'vue';
import { CalendarDate, parseDate, today, getLocalTimeZone, type DateValue } from '@internationalized/date';
import { CalendarDays, Check, ChevronLeft, ChevronRight, X } from 'lucide-vue-next';
import {
  PopoverRoot, PopoverTrigger, PopoverPortal, PopoverContent, PopoverClose,
  CalendarRoot, CalendarGrid, CalendarGridHead, CalendarGridBody, CalendarGridRow,
  CalendarHeadCell, CalendarCell, CalendarCellTrigger,
} from 'reka-ui';
import type { Session } from '@/bindings';
import { groupSessionsByDate, sessionDateParts } from '@/lib/sessionDates';

const props = defineProps<{ session: Session; sessions: Session[] }>();
const emit = defineEmits<{ select: [session: Session] }>();
const open = ref(false);
const view = ref<'days' | 'months' | 'years'>('days');
const pendingDay = shallowRef<DateValue>();
const choicesRef = ref<HTMLElement>();
const sessionsByDay = computed(() => groupSessionsByDate(props.sessions));
const availableDates = computed(() => [...sessionsByDay.value.keys()].sort());
const years = computed(() => [...new Set(availableDates.value.map(key => parseDate(key).year))]);
const availableMonths = computed(() => [...new Set(availableDates.value.map(key => key.slice(0, 7)))].map(key => parseDate(`${key}-01`)));
const currentDate = computed(() => {
  const date = sessionDateParts(props.session.datetime);
  return date ? new CalendarDate(date.year, date.month, date.day) : undefined;
});
const placeholder = shallowRef<DateValue>(currentDate.value ?? today(getLocalTimeZone()));
const calendarValue = computed(() => pendingDay.value ?? currentDate.value);
const dayChoices = computed(() => pendingDay.value ? sessionsByDay.value.get(pendingDay.value.toString()) ?? [] : []);
const previousMonth = computed(() => [...availableMonths.value].reverse().find(date => date.compare(placeholder.value.set({ day: 1 })) < 0));
const nextMonth = computed(() => availableMonths.value.find(date => date.compare(placeholder.value.set({ day: 1 })) > 0));
const monthNames = Array.from({ length: 12 }, (_, index) => new Intl.DateTimeFormat('en-GB', { month: 'short' }).format(new Date(2020, index, 1)));
const headingMonth = computed(() => new Intl.DateTimeFormat('en-GB', { month: 'long' }).format(new Date(2020, placeholder.value.month - 1, 1)));
const pendingLabel = computed(() => pendingDay.value ? new Intl.DateTimeFormat('en-GB', { day: 'numeric', month: 'long', year: 'numeric' }).format(new Date(pendingDay.value.year, pendingDay.value.month - 1, pendingDay.value.day)) : '');
const isUnavailable = (date: DateValue) => !sessionsByDay.value.has(date.toString());
const hasMonth = (month: number) => availableMonths.value.some(date => date.year === placeholder.value.year && date.month === month);
function reset() {
  view.value = 'days';
  pendingDay.value = undefined;
  placeholder.value = currentDate.value ?? (availableDates.value[0] ? parseDate(availableDates.value[0]) : today(getLocalTimeZone()));
}
function chooseSession(session: Session) {
  // Revalidate against current candidates if neighboring columns have changed.
  const candidate = props.sessions.find(candidate => candidate.id === session.id);
  if (!candidate) return;
  emit('select', candidate);
  open.value = false;
}
async function chooseDate(date: DateValue | undefined) {
  const selected = date ?? calendarValue.value;
  if (!selected) return;
  const matches = sessionsByDay.value.get(selected.toString()) ?? [];
  if (matches.length === 1) chooseSession(matches[0]);
  else if (matches.length > 1) {
    pendingDay.value = selected;
    await nextTick();
    choicesRef.value?.querySelector('button')?.focus();
  }
}
function chooseYear(year: number) {
  const month = availableMonths.value.find(date => date.year === year);
  if (!month) return;
  placeholder.value = month;
  view.value = 'months';
}
function chooseMonth(month: number) {
  if (!hasMonth(month)) return;
  placeholder.value = new CalendarDate(placeholder.value.year, month, 1);
  view.value = 'days';
}
function formatTime(datetime: string) {
  return new Intl.DateTimeFormat('en-GB', { hour: '2-digit', minute: '2-digit' }).format(new Date(datetime));
}
watch(open, value => { if (value) reset(); });
watch(() => props.session.id, () => { if (open.value) reset(); });
watch(() => `${placeholder.value.year}-${placeholder.value.month}`, () => { pendingDay.value = undefined; });
</script>

<template>
  <PopoverRoot v-model:open="open">
    <PopoverTrigger :disabled="!availableDates.length" title="Choose session by date" aria-label="Choose session by date"
      class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-gray-300 bg-white text-gray-600 shadow-sm outline-none hover:border-gray-400 hover:bg-gray-50 focus-visible:ring-2 focus-visible:ring-blue-500 disabled:cursor-not-allowed disabled:opacity-40">
      <CalendarDays class="h-4 w-4" />
    </PopoverTrigger>
    <PopoverPortal>
      <PopoverContent align="start" :side-offset="6" :collision-padding="8" aria-label="Choose a session date"
        class="z-[90] w-80 max-w-[calc(100vw-1rem)] max-h-[var(--reka-popover-content-available-height)] overflow-y-auto rounded-xl border border-gray-200 bg-white p-3 text-left text-sm text-gray-700 shadow-xl outline-none">
        <div class="mb-3 flex items-center justify-between">
          <span class="font-semibold text-gray-900">Choose session date</span>
          <PopoverClose aria-label="Close calendar" class="rounded p-1 text-gray-400 hover:bg-gray-100 hover:text-gray-700"><X class="h-4 w-4" /></PopoverClose>
        </div>
        <div class="mb-3 flex items-center justify-between gap-2">
          <button type="button" aria-label="Previous month with sessions" :disabled="!previousMonth" class="rounded p-1.5 hover:bg-gray-100 disabled:opacity-30" @click="placeholder = previousMonth!; view = 'days'"><ChevronLeft class="h-4 w-4" /></button>
          <div class="flex gap-1">
            <button type="button" aria-label="Choose month" :aria-expanded="view === 'months'" class="rounded px-2 py-1 font-semibold hover:bg-blue-50 hover:text-blue-700" @click="view = view === 'months' ? 'days' : 'months'">{{ headingMonth }}</button>
            <button type="button" aria-label="Choose year" :aria-expanded="view === 'years'" class="rounded px-2 py-1 font-semibold hover:bg-blue-50 hover:text-blue-700" @click="view = view === 'years' ? 'days' : 'years'">{{ placeholder.year }}</button>
          </div>
          <button type="button" aria-label="Next month with sessions" :disabled="!nextMonth" class="rounded p-1.5 hover:bg-gray-100 disabled:opacity-30" @click="placeholder = nextMonth!; view = 'days'"><ChevronRight class="h-4 w-4" /></button>
        </div>
        <div v-if="view === 'years'" class="grid grid-cols-3 gap-2" aria-label="Years with selectable sessions">
          <button v-for="year in years" :key="year" type="button" :aria-pressed="year === placeholder.year" class="rounded-lg bg-blue-50 p-3 font-medium text-blue-700 outline-none hover:bg-blue-100 focus-visible:ring-2 focus-visible:ring-blue-500" :class="year === placeholder.year ? 'ring-1 ring-blue-500' : ''" @click="chooseYear(year)">{{ year }}</button>
        </div>
        <div v-else-if="view === 'months'" class="grid grid-cols-3 gap-2" aria-label="Months with selectable sessions">
          <button v-for="(month, index) in monthNames" :key="month" type="button" :disabled="!hasMonth(index + 1)" :aria-pressed="placeholder.month === index + 1" class="rounded-lg p-3 outline-none focus-visible:ring-2 focus-visible:ring-blue-500 disabled:cursor-not-allowed disabled:text-gray-300" :class="hasMonth(index + 1) ? 'bg-blue-50 font-medium text-blue-700 hover:bg-blue-100' : ''" @click="chooseMonth(index + 1)">{{ month }}</button>
        </div>
        <CalendarRoot v-else v-slot="{ grid, weekDays }" v-model:placeholder="placeholder" :model-value="calendarValue" :is-date-unavailable="isUnavailable" :min-value="availableDates[0] ? parseDate(availableDates[0]) : undefined" :max-value="availableDates.length ? parseDate(availableDates[availableDates.length - 1]) : undefined" :week-starts-on="1" locale="en-GB" calendar-label="Selectable session dates" :is-date-disabled="date => date.year !== placeholder.year || date.month !== placeholder.month" @update:model-value="chooseDate">
          <CalendarGrid v-for="month in grid" :key="month.value.toString()" class="w-full table-fixed border-collapse">
            <CalendarGridHead><CalendarGridRow><CalendarHeadCell v-for="day in weekDays" :key="day" class="pb-2 text-center text-xs font-normal text-gray-400">{{ day }}</CalendarHeadCell></CalendarGridRow></CalendarGridHead>
            <CalendarGridBody>
              <CalendarGridRow v-for="(week, index) in month.rows" :key="index">
                <CalendarCell v-for="day in week" :key="day.toString()" :date="day" class="p-0.5 text-center">
                  <CalendarCellTrigger :day="day" :month="month.value" class="relative flex h-9 w-full items-center justify-center rounded-lg outline-none focus-visible:ring-2 focus-visible:ring-blue-500 data-[outside-view]:invisible data-[unavailable]:text-gray-300 data-[disabled]:text-gray-300 data-[selected]:bg-blue-600 data-[selected]:text-white"
                    :class="!isUnavailable(day) ? 'bg-blue-50 font-semibold text-blue-700 hover:bg-blue-100' : ''"
                    :title="!isUnavailable(day) ? `${sessionsByDay.get(day.toString())!.length} ${sessionsByDay.get(day.toString())!.length === 1 ? 'session' : 'sessions'}` : 'No selectable sessions'">
                    {{ day.day }}
                  </CalendarCellTrigger>
                </CalendarCell>
              </CalendarGridRow>
            </CalendarGridBody>
          </CalendarGrid>
        </CalendarRoot>
        <p class="mt-3 text-xs text-gray-500">Blue dates have selectable sessions.</p>
        <div v-if="dayChoices.length > 1 && view === 'days'" ref="choicesRef" class="mt-3 border-t border-gray-100 pt-3">
          <p class="mb-2 text-xs font-medium text-gray-600">{{ pendingLabel }} · Choose a session</p>
          <div class="max-h-48 space-y-1 overflow-y-auto">
            <button v-for="candidate in dayChoices" :key="candidate.id" type="button" :aria-pressed="candidate.id === session.id" class="flex w-full items-center justify-between gap-2 rounded-lg px-3 py-2 text-left outline-none hover:bg-blue-50 focus-visible:ring-2 focus-visible:ring-blue-500" @click="chooseSession(candidate)">
              <span><span class="font-medium">Session #{{ candidate.session_number }}</span><span class="ml-2 text-xs text-gray-500">{{ formatTime(candidate.datetime) }}</span></span>
              <Check v-if="candidate.id === session.id" class="h-4 w-4 text-blue-600" />
            </button>
          </div>
        </div>
      </PopoverContent>
    </PopoverPortal>
  </PopoverRoot>
</template>
