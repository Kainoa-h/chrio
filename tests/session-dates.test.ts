import { describe, expect, test } from 'bun:test';
import { matchesSessionDate, sessionDateParts, type SessionDateFilter } from '../src/lib/sessionDates';

const all: SessionDateFilter = { year: null, month: null, day: null };
const dates = ['2024-02-29T12:00:00', '2025-02-28T12:00:00', '2025-03-28T12:00:00', '2026-02-28T12:00:00'];
const matching = (filter: Partial<SessionDateFilter>) => dates.filter(date => matchesSessionDate(date, { ...all, ...filter }));

describe('session date filters', () => {
  test('each optional date level narrows independently', () => {
    expect(matching({})).toEqual(dates);
    expect(matching({ year: 2025 })).toEqual(dates.slice(1, 3));
    expect(matching({ month: 2 })).toEqual([dates[0], dates[1], dates[3]]);
    expect(matching({ day: 28 })).toEqual(dates.slice(1));
    expect(matching({ year: 2025, month: 2 })).toEqual([dates[1]]);
    expect(matching({ year: 2025, day: 28 })).toEqual(dates.slice(1, 3));
    expect(matching({ month: 2, day: 28 })).toEqual([dates[1], dates[3]]);
    expect(matching({ year: 2025, month: 2, day: 28 })).toEqual([dates[1]]);
  });

  test('handles leap days, no matches, and clearing individual levels', () => {
    expect(matching({ month: 2, day: 29 })).toEqual([dates[0]]);
    expect(matching({ year: 2025, month: 2, day: 29 })).toEqual([]);
    expect(matching({ month: 2, day: 31 })).toEqual([]);
    expect(matching({ year: null, month: 2, day: 28 })).toEqual([dates[1], dates[3]]);
    expect(matching(all)).toEqual(dates);
  });

  test('uses the displayed local date for timestamps near midnight', () => {
    const datetime = '2025-12-31T23:30:00-08:00';
    const date = new Date(datetime);
    const displayed = { year: date.getFullYear(), month: date.getMonth() + 1, day: date.getDate() };
    expect(sessionDateParts(datetime)).toEqual(displayed);
    expect(matchesSessionDate(datetime, displayed)).toBe(true);
    expect(matchesSessionDate(datetime, { ...displayed, year: displayed.year - 1 })).toBe(false);
  });

  test('invalid dates stay visible without filters but cannot match a date filter', () => {
    expect(sessionDateParts('invalid')).toBeNull();
    expect(matchesSessionDate('invalid', all)).toBe(true);
    expect(matchesSessionDate('invalid', { ...all, month: 2 })).toBe(false);
  });
});
