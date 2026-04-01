import { CsvPreview } from '../../types';
import { Button } from '../ui/Button';

interface Props {
  preview: CsvPreview;
  filePath: string;
  onBack: () => void;
  onNext: () => void;
}

export function PreviewStep({ preview, filePath, onBack, onNext }: Props) {
  const filename = filePath.split(/[\\/]/).pop() ?? filePath;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
      <div style={{ fontSize: 13, color: 'var(--text-muted)' }}>
        <span style={{ fontFamily: 'var(--font-mono)' }}>{filename}</span>
        {' '}— {preview.headers.length} columns, delimiter:{' '}
        <code style={{ background: 'var(--bg-hover)', padding: '1px 5px', borderRadius: 3 }}>
          {preview.delimiter === '\t' ? 'tab' : preview.delimiter}
        </code>
      </div>

      <div style={{ overflowX: 'auto', border: '1px solid var(--border-color)', borderRadius: 6 }}>
        <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 12 }}>
          <thead>
            <tr style={{ background: 'var(--bg-sidebar)' }}>
              {preview.headers.map((h, i) => (
                <th key={i} style={{
                  padding: '8px 12px',
                  textAlign: 'left',
                  color: 'var(--text-secondary)',
                  fontWeight: 600,
                  borderBottom: '1px solid var(--border-color)',
                  whiteSpace: 'nowrap',
                }}>
                  {h}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {preview.rows.map((row, ri) => (
              <tr key={ri} style={{ borderBottom: '1px solid var(--border-color)' }}>
                {preview.headers.map((_, ci) => (
                  <td key={ci} style={{
                    padding: '7px 12px',
                    color: 'var(--text-primary)',
                    whiteSpace: 'nowrap',
                    maxWidth: 200,
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                  }}>
                    {row[ci] ?? ''}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div style={{ fontSize: 12, color: 'var(--text-muted)' }}>
        Showing first {preview.rows.length} rows
      </div>

      <div style={{ display: 'flex', justifyContent: 'space-between' }}>
        <Button variant="ghost" onClick={onBack}>← Back</Button>
        <Button variant="primary" onClick={onNext} data-testid="import-preview-btn">Map Columns →</Button>
      </div>
    </div>
  );
}
