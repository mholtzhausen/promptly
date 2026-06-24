/**
 * Parse stored history title `[Name](var1:val1, …)` for display.
 * Format is built by Rust `db::build_history_title`.
 */
export function parseHistoryTitle(title: string): {
  name: string;
  vars: string | null;
} {
  const match = /^\[([^\]]*)\]\((.*)\)$/.exec(title);
  if (!match) return { name: title, vars: null };
  const name = match[1];
  const vars = match[2];
  return { name, vars: vars ? vars : null };
}

export function HistoryTitleText({ title }: { title: string }) {
  const { name, vars } = parseHistoryTitle(title);
  return (
    <span className="history-title">
      <span className="history-title-name">{name}</span>
      {vars !== null && (
        <span className="history-title-vars"> ({vars})</span>
      )}
    </span>
  );
}
