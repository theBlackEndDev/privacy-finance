import { Button } from '../ui/Button';

interface Props {
  data: unknown;
  filename: string;
  format: 'csv' | 'json';
}

export function ReportExportButton({ data, filename, format }: Props) {
  function handleExport() {
    let content: string;
    let mimeType: string;
    let ext: string;

    if (format === 'json') {
      content = JSON.stringify(data, null, 2);
      mimeType = 'application/json';
      ext = 'json';
    } else {
      content = convertToCsv(data);
      mimeType = 'text/csv';
      ext = 'csv';
    }

    const blob = new Blob([content], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${filename}.${ext}`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  return (
    <Button size="sm" variant="ghost" onClick={handleExport}>
      Export {format.toUpperCase()}
    </Button>
  );
}

function convertToCsv(data: unknown): string {
  if (!Array.isArray(data) || data.length === 0) return '';
  const headers = Object.keys(data[0] as Record<string, unknown>);
  const rows = (data as Record<string, unknown>[]).map((row) =>
    headers.map((h) => {
      const val = row[h];
      const str = String(val ?? '');
      return str.includes(',') || str.includes('"') ? `"${str.replace(/"/g, '""')}"` : str;
    }).join(',')
  );
  return [headers.join(','), ...rows].join('\n');
}
