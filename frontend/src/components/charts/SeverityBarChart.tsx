import { Bar, BarChart, CartesianGrid, Cell, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts';

interface Point {
  severity: string;
  count: number;
}

const SEVERITY_ORDER = ['critical', 'high', 'medium', 'low', 'info'];

export default function SeverityBarChart({ data, onBarClick, selectedSeverity }: { data: Point[]; onBarClick?: (p: Point) => void; selectedSeverity?: string | null; }) {
  const sorted = [...data].sort((a, b) => {
    const ia = SEVERITY_ORDER.indexOf(a.severity.toLowerCase());
    const ib = SEVERITY_ORDER.indexOf(b.severity.toLowerCase());
    return (ia === -1 ? 99 : ia) - (ib === -1 ? 99 : ib);
  });

  const colors: Record<string, string> = {
    critical: '#ef4444',
    high: '#f97316',
    medium: '#f59e0b',
    low: '#10b981',
    info: '#3b82f6',
  };

  return (
    <div style={{ width: '100%', height: 320 }}>
      <ResponsiveContainer>
        <BarChart data={sorted} margin={{ top: 10, right: 12, left: 0, bottom: 6 }}>
          <CartesianGrid strokeDasharray="3 3" strokeOpacity={0.06} />
          <XAxis dataKey="severity" tickFormatter={(v: any) => String(v).charAt(0).toUpperCase() + String(v).slice(1)} />
          <YAxis allowDecimals={false} />
          <Tooltip formatter={(value: any) => [value, 'Issues']} />
          <Bar dataKey="count" onClick={(e: any) => onBarClick && onBarClick(e)} radius={[6, 6, 6, 6]}>
            {sorted.map((row, idx) => {
              const sev = row.severity.toLowerCase();
              const isSelected = selectedSeverity && selectedSeverity === sev;
              return (
                <Cell
                  key={`c-${idx}`}
                  fill={colors[sev] || '#64748b'}
                  fillOpacity={isSelected ? 1 : 0.85}
                  stroke={isSelected ? '#111827' : undefined}
                  strokeWidth={isSelected ? 2 : 0}
                />
              );
            })}
          </Bar>
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}
