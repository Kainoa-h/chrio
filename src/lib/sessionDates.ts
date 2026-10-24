export type SessionDateFilter = { year: number | null; month: number | null; day: number | null };

export function sessionDateParts(datetime: string) {
  const date = new Date(datetime);
  if (Number.isNaN(date.getTime())) return null;
  // Use local calendar dates, matching the sessions table's date formatter.
  return { year: date.getFullYear(), month: date.getMonth() + 1, day: date.getDate() };
}

export function matchesSessionDate(datetime: string, filter: SessionDateFilter): boolean {
  if (filter.year === null && filter.month === null && filter.day === null) return true;
  const parts = sessionDateParts(datetime);
  return parts !== null &&
    (filter.year === null || parts.year === filter.year) &&
    (filter.month === null || parts.month === filter.month) &&
    (filter.day === null || parts.day === filter.day);
}
