import type { HistoryEntry } from "../types";
import { HistoryTitleText } from "../lib/historyTitle";

type HistoryDetailViewProps = {
  historyDetail: HistoryEntry;
  historyDetailContent: string;
  setHistoryDetailContent: (content: string) => void;
  onClose: () => void;
  onCopy: () => void;
};

export function HistoryDetailView({
  historyDetail,
  historyDetailContent,
  setHistoryDetailContent,
  onClose,
  onCopy,
}: HistoryDetailViewProps) {
  return (
    <div className="app history-detail-view">
      <h1 className="panel-header">
        <HistoryTitleText title={historyDetail.title} />
      </h1>
      <div className="history-detail-body">
        {historyDetail.variables.length > 0 && (
          <div className="history-variables">
            <table className="history-variables-table">
              <thead>
                <tr>
                  <th>Variable</th>
                  <th>Value</th>
                </tr>
              </thead>
              <tbody>
                {historyDetail.variables.map((v) => (
                  <tr key={v.name}>
                    <td className="history-var-name">{v.name}</td>
                    <td className="history-var-value mono">{v.value}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
        <label className="preview-field history-content-field">
          Resulting prompt
          <textarea
            className="mono multiline preview"
            value={historyDetailContent}
            onChange={(e) => setHistoryDetailContent(e.target.value)}
          />
        </label>
      </div>
      <div className="history-detail-footer panel-footer">
        <div className="buttons">
          <button type="button" onClick={onClose}>
            Close
          </button>
          <button type="button" className="primary" onClick={onCopy}>
            Copy
          </button>
        </div>
      </div>
    </div>
  );
}
