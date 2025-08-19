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

  const handleKeyDown = (e: React.KeyboardEvent, point: Point) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      onBarClick && onBarClick(point);
    }
  };

  return (
    <div 
      style={{ width: '100%', height: 320 }}
      role="application"
      aria-label={`Interactive bar chart showing issues by severity. Use Enter or Space to filter. ${sorted.map(item => `${item.severity}: ${item.count} issues`).join(', ')}`}
      tabIndex={0}
      onKeyDown={(e) => {
        if ((e.key === 'Enter' || e.key === ' ') && sorted.length > 0) {
          e.preventDefault();
          // Default to first item if none selected
          const targetSeverity = selectedSeverity || sorted[0].severity;
          const point = sorted.find(p => p.severity.toLowerCase() === targetSeverity.toLowerCase()) || sorted[0];
          onBarClick && onBarClick(point);
        }
      }}
    >
      <ResponsiveContainer>
        <BarChart 
          data={sorted} 
          margin={{ top: 10, right: 12, left: 0, bottom: 6 }}
          aria-label="Issues by severity chart"
        >
          <CartesianGrid strokeDasharray="3 3" strokeOpacity={0.06} />
          <XAxis 
            dataKey="severity" 
            tickFormatter={(v: any) => String(v).charAt(0).toUpperCase() + String(v).slice(1)} 
          />
          <YAxis allowDecimals={false} />
          <Tooltip 
            formatter={(value: any) => [value, 'Issues']} 
            labelFormatter={(severity: any) => `${String(severity).charAt(0).toUpperCase() + String(severity).slice(1)} Severity`}
          />
          <Bar 
            dataKey="count" 
            onClick={(e: any) => onBarClick && onBarClick(e)} 
            radius={[6, 6, 6, 6]}
            aria-label="Click bars to filter issues by severity"
          >
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
